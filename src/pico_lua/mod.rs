// Hand-rolled Lua subset interpreter for PICO-8 carts.
//
// Design choices:
//  - Tree-walking, not bytecode. Cart code runs once per frame for _update +
//    _draw; throughput is dwarfed by graphics work.
//  - No coroutines (carts using flip() won't work; the dominant
//    _init/_update/_draw pattern does).
//  - Numbers are f64. PICO-8's 16:16 fixed-point semantics are emulated only
//    where they leak into the API surface (band/bor/shl/etc, peek4/poke4,
//    tostr/tonum hex flags). Pure-arithmetic carts won't notice, but
//    overflow-trick carts will.

pub mod ast;
pub mod lex;
pub mod parse;
pub mod value;
pub mod interp;
pub mod api;
pub mod serialize;

use std::cell::RefCell;
use std::rc::Rc;

use ast::Block;
use interp::Interp;
use value::*;

use crate::cart::Cart;
use crate::lua_engine::LuaEngine;
use crate::preprocessor;
use crate::state::PicoState;

pub struct LuaImpl {
    interp: Interp,
    chunk: Option<Block>,
    has_init: bool,
    has_update: bool,
    has_update60: bool,
    has_draw: bool,
    error: Option<String>,
}

impl LuaImpl {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(TableInner::new()));
        api::register_all(&globals);

        // Install _G as a self-reference
        globals.borrow_mut().set(Value::Str(Rc::from(b"_G".as_slice())), Value::Table(Rc::clone(&globals)));

        let interp = Interp::new(globals);
        Self {
            interp,
            chunk: None,
            has_init: false,
            has_update: false,
            has_update60: false,
            has_draw: false,
            error: None,
        }
    }

    fn detect_lifecycle(&mut self) {
        let g = self.interp.globals.borrow();
        let is_fn = |name: &[u8]| matches!(g.get(&Value::Str(Rc::from(name))), Value::Function(_));
        self.has_init = is_fn(b"_init");
        self.has_update = is_fn(b"_update");
        self.has_update60 = is_fn(b"_update60");
        self.has_draw = is_fn(b"_draw");
    }

    fn call_global(&mut self, name: &[u8]) {
        if self.error.is_some() { return; }
        let f = self.interp.globals.borrow().get(&Value::Str(Rc::from(name)));
        if matches!(f, Value::Nil) { return; }
        if let Err(e) = self.interp.call_value(&f, Vec::new()) {
            let msg = format!(
                "{}:{} in {}: {}",
                self.interp.chunkname,
                self.interp.current_line,
                String::from_utf8_lossy(name),
                format_err(&e)
            );
            self.error = Some(msg);
        }
    }
}

fn format_err(e: &RtError) -> String {
    match &e.value {
        Value::Str(s) => String::from_utf8_lossy(s).into_owned(),
        v => format!("{:?}", v),
    }
}

impl LuaEngine for LuaImpl {
    fn load_cart(&mut self, state: &mut PicoState, cart: &Cart) -> Result<(), String> {
        // Preprocess PICO-8 source to standard Lua 5.2
        let processed = preprocessor::preprocess(&cart.lua_code);

        // Tokenize + parse
        let bytes = processed.as_bytes();
        let mut lex = lex::Lexer::new(bytes);
        let toks = lex.tokenize().map_err(|e| format!("lex error line {}: {}", e.line, e.msg))?;
        let mut p = parse::Parser::new(toks);
        let chunk = p.parse_chunk().map_err(|e| format!("parse error line {}: {}", e.line, e.msg))?;

        // Execute top-level (defines _init/_update/_draw and any module-scope code)
        self.interp.host = state as *mut PicoState;
        self.error = None;

        // Top-level block: wrap in a Lua function so `return` works at chunk level
        // and varargs are bound to the top frame as empty.
        self.interp.frames.push(interp::Frame {
            locals: Vec::new(),
            varargs: Vec::new(),
            upvalues: Vec::new(),
        });

        // Install the env-fallback shim before user code runs (so `all()` results
        // used as _ENV resolve globals via __index=_G). The original PICO-Z does
        // this via Lua source; we replicate it.
        let shim = b"do local mt={__index=_G} local rawall=all local rawforeach=foreach \
            local sm=setmetatable local gm=getmetatable local tp=type \
            local function wrap(v) if tp(v)==\"table\" and not gm(v) then sm(v,mt) end return v end \
            all=function(t) local it=rawall(t) return function(s,c) return wrap(it(s,c)) end,t end \
            foreach=function(t,f) return rawforeach(t,function(v) return f(wrap(v)) end) end end";
        if let Err(e) = run_str(&mut self.interp, shim) {
            // shim failure shouldn't abort cart load
            let _ = e;
        }

        let result = self.interp.execute_block(&chunk);
        self.interp.frames.pop();
        self.interp.host = std::ptr::null_mut();

        match result {
            Ok(_) => {
                self.chunk = Some(chunk);
                self.detect_lifecycle();
                Ok(())
            }
            Err(e) => {
                let msg = format_err(&e);
                self.error = Some(msg.clone());
                Err(msg)
            }
        }
    }

    fn call_init(&mut self, state: &mut PicoState) {
        self.interp.host = state as *mut PicoState;
        if self.has_init {
            self.call_global(b"_init");
        }
        self.interp.host = std::ptr::null_mut();
    }
    fn call_update(&mut self, state: &mut PicoState) {
        self.interp.host = state as *mut PicoState;
        if self.has_update60 { self.call_global(b"_update60"); }
        else if self.has_update { self.call_global(b"_update"); }
        self.interp.host = std::ptr::null_mut();
    }
    fn call_draw(&mut self, state: &mut PicoState) {
        self.interp.host = state as *mut PicoState;
        if self.has_draw { self.call_global(b"_draw"); }
        self.interp.host = std::ptr::null_mut();
    }
    fn use_60fps(&self) -> bool { self.has_update60 }
    fn had_error(&self) -> bool { self.error.is_some() }
    fn error_message(&self) -> &str { self.error.as_deref().unwrap_or("") }
    fn save_globals(&self) -> Vec<u8> {
        serialize::save_globals(&self.interp.globals)
    }
    fn load_globals(&mut self, data: &[u8]) -> Result<(), String> {
        serialize::load_globals(&self.interp.globals, data)
    }
}

impl LuaImpl {
    /// Lossless in-session snapshot of Lua state. Deep-clones the globals
    /// table tree so subsequent cart code can't mutate the snapshot.
    /// Closures inside tables are preserved by Rc-sharing (they're code).
    pub fn snapshot(&self) -> Rc<value::Table> {
        let mut seen = std::collections::HashMap::new();
        value::deep_clone_table(&self.interp.globals, &mut seen)
    }

    /// Restore Lua state from a previously-taken snapshot, deep-cloning
    /// again so further saves/loads stay independent.
    pub fn restore(&mut self, snap: &Rc<value::Table>) {
        let mut seen = std::collections::HashMap::new();
        let restored = value::deep_clone_table(snap, &mut seen);
        // Replace globals contents in place so any references held elsewhere
        // (none currently, but defensive) keep pointing at the right table.
        let mut g = self.interp.globals.borrow_mut();
        let r = restored.borrow();
        g.map.clear();
        for (k, v) in r.map.iter() {
            g.map.insert(k.clone(), v.clone());
        }
        g.metatable = r.metatable.clone();
        g.array_max_hint = r.array_max_hint;
    }
}

fn run_str(interp: &mut Interp, src: &[u8]) -> Result<(), RtError> {
    let mut lx = lex::Lexer::new(src);
    let toks = lx.tokenize().map_err(|e| RtError::msg(format!("shim lex: {} (line {})", e.msg, e.line)))?;
    let mut p = parse::Parser::new(toks);
    let block = p.parse_chunk().map_err(|e| RtError::msg(format!("shim parse: {} (line {})", e.msg, e.line)))?;
    interp.execute_block(&block).map(|_| ())
}
