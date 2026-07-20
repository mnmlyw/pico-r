// wasm32-unknown-unknown supports std with minimal runtime; using std avoids
// the alloc/no_std import dance and is fine for binary-size targets.

#![allow(dead_code)] // Many fields/funcs are exposed for the future Lua API.
#![deny(unsafe_op_in_unsafe_fn)]

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;

pub mod audio;
pub mod cart;
pub mod gfx;
pub mod gfx_font;
pub mod input;
pub mod lua_engine;
pub mod memory;
pub mod palette;
pub mod pico_lua;
pub mod preprocessor;
pub mod state;
pub mod trigtables;

use lua_engine::LuaEngine;
use pico_lua::LuaImpl;
use state::{PicoState, SCREEN_H, SCREEN_W};

unsafe fn wasm_bytes<'a>(ptr: *const u8, len: u32) -> Option<&'a [u8]> {
    let len = len as usize;
    if len == 0 {
        return Some(&[]);
    }
    if ptr.is_null() {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts(ptr, len) })
}

// === Global allocator ===
//
// The `dlmalloc` crate would normally be used; for simplicity we use
// `lol_alloc`-style approach with the linked-list allocator. To keep the
// dependency graph tiny, here we use a small bump allocator. Real production
// use should switch to `dlmalloc` or `wee_alloc` (or a system allocator).
//
// For now we rely on Rust's default allocator on wasm32-unknown-unknown,
// which is `dlmalloc` (built into rustc since 1.77).

// === Single global engine state ===
//
// WASM is single-threaded, so a static UnsafeCell is safe.
struct EngineCell {
    inner: UnsafeCell<MaybeUninit<Engine>>,
    initialized: UnsafeCell<bool>,
}
unsafe impl Sync for EngineCell {}

struct Engine {
    state: PicoState,
    lua: LuaImpl,
    save_data: Option<Vec<u8>>,
    audio_buf: [f32; 4096],
    /// Lossless in-memory snapshot for same-session save/load (P/L).
    /// Captures everything the byte-serialized form drops (closures inside
    /// tables, audio channels, frame state). Cleared on cart load.
    snapshot: Option<Snapshot>,
}

struct Snapshot {
    ram: Box<[u8; memory::RAM_SIZE]>,
    rom: Box<[u8; memory::RAM_SIZE]>,
    audio_channels: [audio::Channel; audio::NUM_CHANNELS],
    audio_music: audio::MusicState,
    audio_noise_seed: u32,
    rng_hi: u32,
    rng_lo: u32,
    elapsed_time: f64,
    frame_count: u32,
    target_fps: u8,
    line_x: i32,
    line_y: i32,
    line_valid: bool,
    lua_globals: std::rc::Rc<pico_lua::value::Table>,
}

static ENGINE: EngineCell = EngineCell {
    inner: UnsafeCell::new(MaybeUninit::uninit()),
    initialized: UnsafeCell::new(false),
};

#[inline]
fn engine_mut() -> Option<&'static mut Engine> {
    unsafe {
        if !*ENGINE.initialized.get() {
            return None;
        }
        Some(&mut *(*ENGINE.inner.get()).as_mut_ptr())
    }
}

fn engine_init() -> &'static mut Engine {
    unsafe {
        let ptr = (*ENGINE.inner.get()).as_mut_ptr();
        ptr.write(Engine {
            state: PicoState::new(),
            lua: LuaImpl::new(),
            save_data: None,
            audio_buf: [0.0; 4096],
            snapshot: None,
        });
        *ENGINE.initialized.get() = true;
        &mut *ptr
    }
}

// === Panic capture ===
//
// With `panic = "abort"`, a panic traps the WASM module and the JS side gets
// no message — just "unreachable executed". Install a hook that writes the
// panic location + message into a static buffer the JS side can read via
// `web_get_panic_msg` / `web_get_panic_len`.

const PANIC_BUF_SIZE: usize = 1024;

struct PanicBuf {
    inner: UnsafeCell<[u8; PANIC_BUF_SIZE]>,
    len: UnsafeCell<usize>,
}

unsafe impl Sync for PanicBuf {}

static PANIC_BUF: PanicBuf = PanicBuf {
    inner: UnsafeCell::new([0; PANIC_BUF_SIZE]),
    len: UnsafeCell::new(0),
};

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let s = format!("{info}");
        let bytes = s.as_bytes();
        let n = bytes.len().min(PANIC_BUF_SIZE);
        // Safety: WASM is single-threaded; only the panicking thread writes here.
        unsafe {
            let buf = &mut *PANIC_BUF.inner.get();
            buf[..n].copy_from_slice(&bytes[..n]);
            *PANIC_BUF.len.get() = n;
        }
    }));
}

// === Exported WASM API ===

/// Install the WASM panic hook so panics surface a readable message instead
/// of a bare `unreachable` trap. JS should call this once on startup.
#[no_mangle]
pub extern "C" fn web_init_panic_hook() {
    install_panic_hook();
}

/// Pointer to the captured panic message buffer.
#[no_mangle]
pub extern "C" fn web_get_panic_msg() -> *const u8 {
    PANIC_BUF.inner.get() as *const u8
}

/// Length of the captured panic message (0 if no panic has occurred).
#[no_mangle]
pub extern "C" fn web_get_panic_len() -> u32 {
    // Safety: WASM is single-threaded.
    unsafe { *PANIC_BUF.len.get() as u32 }
}

#[no_mangle]
pub extern "C" fn web_alloc(len: u32) -> *mut u8 {
    let len = len as usize;
    let mut buf = Vec::new();
    if buf.try_reserve_exact(len).is_err() {
        return std::ptr::null_mut();
    }
    buf.resize(len, 0);
    Box::leak(buf.into_boxed_slice()).as_mut_ptr()
}

/// # Safety
///
/// `ptr` must be a pointer returned by `web_alloc` for the same `len`.
#[no_mangle]
pub unsafe extern "C" fn web_free(ptr: *mut u8, len: u32) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let raw = std::ptr::slice_from_raw_parts_mut(ptr, len as usize);
        let _ = Box::from_raw(raw);
    }
}

/// Initialize emulator with cart data. Returns 0 on success, 1 on error.
///
/// # Safety
///
/// `cart_ptr` must point to `cart_len` readable bytes, unless `cart_len` is 0.
#[no_mangle]
pub unsafe extern "C" fn web_init(cart_ptr: *const u8, cart_len: u32) -> u32 {
    let data = match unsafe { wasm_bytes(cart_ptr, cart_len) } {
        Some(data) => data,
        None => return 1,
    };

    let engine = if let Some(e) = engine_mut() {
        e.state = PicoState::new();
        e.lua = LuaImpl::new();
        e.save_data = None;
        e.snapshot = None;
        e
    } else {
        engine_init()
    };

    let cart = match cart::load_bytes(data, &mut engine.state.memory) {
        Ok(c) => c,
        Err(_) => return 1,
    };
    engine.state.memory.save_rom();
    engine.state.prepare_for_cart_load();

    if engine.lua.load_cart(&mut engine.state, &cart).is_err() {
        return 1;
    }
    engine.lua.call_init(&mut engine.state);
    0
}

/// Run one frame: input→update→draw→render-to-pixel-buffer.
#[no_mangle]
pub extern "C" fn web_update() {
    let engine = match engine_mut() {
        Some(e) => e,
        None => return,
    };

    engine.state.input.mouse_wheel = 0;
    engine.state.input.update();
    engine.state.memory.ram[memory::ADDR_INPUT_P0 as usize] = engine.state.input.btn_state[0];
    engine.state.memory.ram[memory::ADDR_INPUT_P1 as usize] = engine.state.input.btn_state[1];

    // Confirmed against official PICO-8: elapsed_time/frame_count tick up
    // BEFORE this frame's _update/_draw run, not after -- see
    // elapsed-time-ticks-before-not-after-frame-callbacks.
    engine.state.frame_count += 1;
    engine.state.target_fps = if engine.lua.use_60fps() { 60 } else { 30 };
    engine.state.elapsed_time += 1.0 / engine.state.target_fps as f64;

    engine.lua.call_update(&mut engine.state);
    engine.lua.call_draw(&mut engine.state);

    // If the Lua engine has an error, paint it on the screen so it's visible.
    if engine.lua.had_error() {
        let msg = engine.lua.error_message().to_string();
        gfx::cls(&mut engine.state, 0);
        let _ = gfx::draw_text(&mut engine.state.memory, b"LUA ERROR:", 2, 2, 8, false);
        let bytes = msg.as_bytes();
        let mut y = 12i32;
        let mut start = 0usize;
        // Word-wrap to 30 chars (~120px)
        while start < bytes.len() && y < 120 {
            let end = (start + 30).min(bytes.len());
            let _ = gfx::draw_text(&mut engine.state.memory, &bytes[start..end], 2, y, 7, false);
            start = end;
            y += 7;
        }
    }

    // Force borrow split: we need a mutable ref to pixel_buffer and an immutable
    // ref to memory. Both live in PicoState — pull pointers.
    let mem_ref: &memory::Memory = &engine.state.memory;
    let buf: &mut [u32; SCREEN_W * SCREEN_H] = &mut engine.state.pixel_buffer;
    gfx::render_to_argb(mem_ref, buf);
}

#[no_mangle]
pub extern "C" fn web_get_pixel_buffer() -> *const u32 {
    match engine_mut() {
        Some(e) => e.state.pixel_buffer.as_ptr(),
        None => std::ptr::null(),
    }
}

/// Read a single RAM byte -- exposes nothing a cart can't already read via
/// its own `peek()`, but lets an external headless harness (see
/// tools/wasm-test/) inspect engine state (palette remap, screen memory,
/// draw registers, ...) directly against the real `pico_r.wasm` artifact
/// without needing a browser.
#[no_mangle]
pub extern "C" fn web_debug_peek(addr: u32) -> u8 {
    match engine_mut() {
        Some(e) => e.state.memory.ram[(addr as usize) & 0xFFFF],
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn web_set_buttons(player: u8, buttons: u8) {
    if let Some(e) = engine_mut() {
        if (player as usize) < 2 {
            e.state.input.btn_state[player as usize] = buttons;
        }
    }
}

#[no_mangle]
pub extern "C" fn web_set_mouse(x: i32, y: i32, buttons: u8, wheel: i32) {
    if let Some(e) = engine_mut() {
        e.state.input.mouse_x = x;
        e.state.input.mouse_y = y;
        e.state.input.mouse_buttons = buttons;
        e.state.input.mouse_wheel = wheel;
    }
}

#[no_mangle]
pub extern "C" fn web_generate_audio(sample_count: u32) -> *mut f32 {
    let engine = match engine_mut() {
        Some(e) => e,
        None => return std::ptr::null_mut(),
    };
    let count = (sample_count as usize).min(engine.audio_buf.len());
    for i in 0..count {
        engine.audio_buf[i] = engine.state.audio.generate_sample(&engine.state.memory);
    }
    engine.audio_buf.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn web_get_fps() -> u32 {
    engine_mut()
        .map(|e| e.state.target_fps as u32)
        .unwrap_or(30)
}

#[no_mangle]
pub extern "C" fn web_has_error() -> u32 {
    engine_mut()
        .map(|e| if e.lua.had_error() { 1 } else { 0 })
        .unwrap_or(0)
}

// === Save state API ===

#[no_mangle]
pub extern "C" fn web_save_state() -> u32 {
    let engine = match engine_mut() {
        Some(e) => e,
        None => return 0,
    };

    // (1) Lossless in-memory snapshot — used for same-session restore.
    let mut ram = Box::new([0u8; memory::RAM_SIZE]);
    ram.copy_from_slice(&engine.state.memory.ram);
    let mut rom = Box::new([0u8; memory::RAM_SIZE]);
    rom.copy_from_slice(&engine.state.memory.rom);
    engine.snapshot = Some(Snapshot {
        ram,
        rom,
        audio_channels: engine.state.audio.channels.clone(),
        audio_music: engine.state.audio.music_state.clone(),
        audio_noise_seed: engine.state.audio.noise_seed,
        rng_hi: engine.state.rng_hi,
        rng_lo: engine.state.rng_lo,
        elapsed_time: engine.state.elapsed_time,
        frame_count: engine.state.frame_count,
        target_fps: engine.state.target_fps,
        line_x: engine.state.line_x,
        line_y: engine.state.line_y,
        line_valid: engine.state.line_valid,
        lua_globals: engine.lua.snapshot(),
    });

    // (2) Lossy byte-serialized form — for cross-session localStorage backup.
    // Format: [magic "PSAV"][u32 lua_blob_len][lua blob][RAM 65536][ROM 65536]
    let lua_blob = engine.lua.save_globals();
    let mut buf: Vec<u8> = Vec::with_capacity(memory::RAM_SIZE * 2 + lua_blob.len() + 16);
    buf.extend_from_slice(b"PSAV");
    buf.extend_from_slice(&(lua_blob.len() as u32).to_le_bytes());
    buf.extend_from_slice(&lua_blob);
    buf.extend_from_slice(&engine.state.memory.ram);
    buf.extend_from_slice(&engine.state.memory.rom);
    let len = match u32::try_from(buf.len()) {
        Ok(len) => len,
        Err(_) => return 0,
    };
    engine.save_data = Some(buf);
    len
}

#[no_mangle]
pub extern "C" fn web_get_save_ptr() -> *const u8 {
    engine_mut()
        .and_then(|e| e.save_data.as_ref().map(|d| d.as_ptr()))
        .unwrap_or(std::ptr::null())
}

#[no_mangle]
pub extern "C" fn web_free_save() {
    if let Some(e) = engine_mut() {
        e.save_data = None;
    }
}

/// # Safety
///
/// `data_ptr` must point to `data_len` readable bytes when no same-session
/// in-memory snapshot is available, unless `data_len` is 0.
#[no_mangle]
pub unsafe extern "C" fn web_load_state(data_ptr: *const u8, data_len: u32) -> u32 {
    let engine = match engine_mut() {
        Some(e) => e,
        None => return 1,
    };

    // Prefer the lossless in-memory snapshot (same-session save/load).
    if let Some(snap) = &engine.snapshot {
        engine.state.memory.ram.copy_from_slice(snap.ram.as_ref());
        engine.state.memory.rom.copy_from_slice(snap.rom.as_ref());
        engine.state.audio.channels = snap.audio_channels.clone();
        engine.state.audio.music_state = snap.audio_music.clone();
        engine.state.audio.noise_seed = snap.audio_noise_seed;
        engine.state.rng_hi = snap.rng_hi;
        engine.state.rng_lo = snap.rng_lo;
        engine.state.elapsed_time = snap.elapsed_time;
        engine.state.frame_count = snap.frame_count;
        engine.state.target_fps = snap.target_fps;
        engine.state.line_x = snap.line_x;
        engine.state.line_y = snap.line_y;
        engine.state.line_valid = snap.line_valid;
        engine.lua.restore(&snap.lua_globals);
        return 0;
    }

    // Cross-session fallback: deserialize bytes from localStorage. This path
    // drops closures stored in tables — they'll be missing methods until the
    // cart's code reattaches them. Use only when no in-memory snapshot exists.
    let data = match unsafe { wasm_bytes(data_ptr, data_len) } {
        Some(data) => data,
        None => return 1,
    };
    if data.len() < 8 || &data[0..4] != b"PSAV" {
        return 1;
    }
    let lua_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let lua_start: usize = 8;
    let Some(lua_end) = lua_start.checked_add(lua_len) else {
        return 1;
    };
    let Some(ram_end) = lua_end.checked_add(memory::RAM_SIZE) else {
        return 1;
    };
    let Some(rom_end) = ram_end.checked_add(memory::RAM_SIZE) else {
        return 1;
    };
    if data.len() < rom_end {
        return 1;
    }
    if engine.lua.load_globals(&data[lua_start..lua_end]).is_err() {
        return 1;
    }
    engine
        .state
        .memory
        .ram
        .copy_from_slice(&data[lua_end..ram_end]);
    engine
        .state
        .memory
        .rom
        .copy_from_slice(&data[ram_end..rom_end]);
    0
}
