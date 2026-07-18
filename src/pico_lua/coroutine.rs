// Coroutines for the tree-walking interpreter, implemented as OS threads
// with strict baton-passing: at any instant exactly ONE thread (the main
// interpreter thread or one coroutine thread) is executing interpreter
// code; everyone else is blocked on a channel recv. Every handoff is a
// channel send/recv pair, which establishes the happens-before edge that
// makes sharing the (Rc/RefCell-based, !Send) interpreter state sound in
// practice -- values (including Rc reference counts) are only ever
// touched by whichever thread currently holds the baton.
//
// The interpreter's single `frames` stack is swapped at each handoff (the
// resumer stashes its own frames and installs the coroutine's saved
// frames; yield/return do the reverse), so each side always sees exactly
// its own Lua call stack while running.
//
// A coroutine handle that is discarded while suspended leaks its parked
// thread until process exit -- acceptable for the headless runner; the
// wasm build (which has no std::thread) compiles this module out and
// cocreate reports an error instead.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;

use super::interp::{Frame, Interp};
use super::value::Value;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoStatus {
    Suspended,
    Running,
    Dead,
}

pub enum CoEvent {
    Yield(Vec<Value>),
    Return(Vec<Value>),
    Error(String),
}

/// Wrapper asserting Send for baton-passed payloads. Sound only under the
/// strict handoff discipline described in the module docs.
struct Baton<T>(T);
unsafe impl<T> Send for Baton<T> {}

impl<T> Baton<T> {
    /// Unwrap through a method call (NOT a pattern destructure) -- edition
    /// 2021's disjoint closure capture would otherwise capture the inner
    /// fields individually, stripping the Send assertion.
    fn into_inner(self) -> T {
        self.0
    }
}

pub struct CoroutineHandle {
    /// main -> coroutine: resume arguments. Sender used by the main
    /// thread; Receiver used exclusively by the coroutine thread.
    resume_tx: mpsc::Sender<Baton<Vec<Value>>>,
    resume_rx: mpsc::Receiver<Baton<Vec<Value>>>,
    /// coroutine -> main: yield/return/error events. Sender used
    /// exclusively by the coroutine thread; Receiver by the main thread.
    event_tx: mpsc::Sender<Baton<CoEvent>>,
    event_rx: mpsc::Receiver<Baton<CoEvent>>,
    pub status: Cell<CoStatus>,
    /// The coroutine's paused Lua frame stack while it is suspended.
    saved_frames: RefCell<Vec<Frame>>,
}

// SAFETY / lifetime constraint: the raw Interp pointer captured at create
// time is dereferenced on the coroutine thread at every resume, and the
// coroutine's paused Rust recursion holds it across suspensions -- the
// Interp (and thus the LuaImpl that owns it) must not move in memory for
// the lifetime of any live coroutine. The native hosts keep LuaImpl in
// one place for the whole run; wasm has no threads (spawn fails and this
// returns Err, surfacing as a Lua error from cocreate).
pub fn cocreate(interp: &mut Interp, func: Value) -> Result<Rc<CoroutineHandle>, String> {
    let (resume_tx, resume_rx) = mpsc::channel::<Baton<Vec<Value>>>();
    let (event_tx, event_rx) = mpsc::channel::<Baton<CoEvent>>();
    let handle = Rc::new(CoroutineHandle {
        resume_tx,
        resume_rx,
        event_tx,
        event_rx,
        status: Cell::new(CoStatus::Suspended),
        saved_frames: RefCell::new(Vec::new()),
    });

    let interp_ptr = interp as *mut Interp as usize;
    let payload = Baton((interp_ptr, func, Rc::clone(&handle)));
    let spawned = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let (interp_addr, func, co) = payload.into_inner();
            // Block until the first resume hands us the baton. (Touching
            // co's channel ends before then is safe: the handle was fully
            // constructed before spawn, and mpsc recv is the sync point.)
            let Ok(Baton(first_args)) = co.resume_rx.recv() else {
                // Never resumed and the interpreter dropped the sender side
                // -- can't happen while the handle itself keeps it alive,
                // so this is process-teardown; just exit without touching
                // shared state.
                std::mem::forget(co);
                std::mem::forget(func);
                return;
            };
            // We now hold the baton; the resumer is blocked on event_rx
            // and has installed our (empty) frame stack.
            let interp: &mut Interp = unsafe { &mut *(interp_addr as *mut Interp) };
            let result = interp.call_value(&func, first_args);
            // Body finished (or errored): restore the resumer's frames,
            // mark dead, hand the baton back one last time.
            let resumer_frames = interp
                .coroutine_resume_ctx
                .pop()
                .expect("coroutine return with no resume context");
            interp.frames = resumer_frames;
            interp.coroutine_current.pop();
            co.status.set(CoStatus::Dead);
            let event = match result {
                Ok(vals) => CoEvent::Return(vals),
                Err(e) => CoEvent::Error(match &e.value {
                    Value::Str(s) => String::from_utf8_lossy(s).into_owned(),
                    v => format!("{:?}", v),
                }),
            };
            // Drop this thread's owned interpreter values BEFORE waking the
            // main thread, so their Rc count mutations stay inside our
            // baton window. The event payload itself crosses via Baton.
            let tx = co.event_tx.clone();
            drop(func);
            drop(co);
            let _ = tx.send(Baton(event));
        });
    match spawned {
        Ok(_) => Ok(handle),
        Err(e) => Err(format!("cocreate: cannot spawn coroutine ({})", e)),
    }
}

/// The resume half of the handoff. Ok(values) for a successful
/// yield/return, Err(message) if the coroutine errored or can't resume.
pub fn coresume(
    interp: &mut Interp,
    co: &Rc<CoroutineHandle>,
    args: Vec<Value>,
) -> Result<Vec<Value>, String> {
    match co.status.get() {
        CoStatus::Dead => return Err("corolib: cannot resume dead coroutine".into()),
        CoStatus::Running => return Err("corolib: cannot resume non-suspended coroutine".into()),
        CoStatus::Suspended => {}
    }
    // Stash our frames, install the coroutine's.
    let mine = std::mem::replace(&mut interp.frames, co.saved_frames.take());
    interp.coroutine_resume_ctx.push(mine);
    interp.coroutine_current.push(Rc::clone(co));
    co.status.set(CoStatus::Running);

    if co.resume_tx.send(Baton(args)).is_err() {
        co.status.set(CoStatus::Dead);
        interp.coroutine_current.pop();
        let mine = interp.coroutine_resume_ctx.pop().unwrap_or_default();
        interp.frames = mine;
        return Err("corolib: cannot resume dead coroutine".into());
    }
    match co.event_rx.recv() {
        Ok(Baton(CoEvent::Yield(vals))) => Ok(vals),
        Ok(Baton(CoEvent::Return(vals))) => Ok(vals),
        Ok(Baton(CoEvent::Error(msg))) => Err(msg),
        Err(_) => {
            co.status.set(CoStatus::Dead);
            Err("coroutine thread terminated unexpectedly".into())
        }
    }
}

/// The yield half of the handoff. Runs ON the coroutine's thread inside
/// api_yield. Blocks until the next resume and returns its arguments.
pub fn coyield(interp: &mut Interp, vals: Vec<Value>) -> Result<Vec<Value>, String> {
    let Some(co) = interp.coroutine_current.pop() else {
        return Err("attempt to yield from outside a coroutine".into());
    };
    // Stash our frames into the handle, restore the resumer's.
    let resumer = interp
        .coroutine_resume_ctx
        .pop()
        .expect("yield with no resume context");
    let mine = std::mem::replace(&mut interp.frames, resumer);
    *co.saved_frames.borrow_mut() = mine;
    co.status.set(CoStatus::Suspended);

    let _ = co.event_tx.send(Baton(CoEvent::Yield(vals)));
    // Baton handed back to the resumer. Block for the next resume. We
    // still hold one Rc clone of the handle, but do not touch its count
    // until we're woken (the recv is the sync point).
    match co.resume_rx.recv() {
        Ok(Baton(args)) => Ok(args),
        Err(_) => Err("coroutine resumer disappeared".into()),
    }
}
