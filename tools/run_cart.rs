// Host-side cart runner — loads a cart, executes a few frames, prints errors.
// Usage: cargo run --bin run-cart -- path/to/cart.p8.png [n_frames]

use pico_r::{cart, lua_engine::LuaEngine, pico_lua::LuaImpl, preprocessor, state::PicoState};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: run-cart <cart> [n_frames]");
        std::process::exit(1);
    }
    let path = &args[1];
    let n_frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(60);

    let data = std::fs::read(path).expect("read cart");
    let mut state = PicoState::new();
    let cart = cart::load_bytes(&data, &mut state.memory).expect("load cart");
    state.memory.save_rom();
    state.prepare_for_cart_load();

    eprintln!("cart loaded; lua source {} bytes", cart.lua_code.len());
    let processed = preprocessor::preprocess(&cart.lua_code);
    eprintln!("preprocessed {} bytes", processed.len());

    let mut lua = LuaImpl::new();
    if let Err(e) = lua.load_cart(&mut state, &cart) {
        eprintln!("LOAD ERROR: {}", e);
        // Dump the relevant part of preprocessed source for context
        eprintln!("---- preprocessed (head) ----");
        for line in processed.lines().take(20) {
            eprintln!("{}", line);
        }
        std::process::exit(2);
    }
    eprintln!("loaded; calling _init");
    lua.call_init(&mut state);
    if lua.had_error() {
        eprintln!("INIT ERROR: {}", lua.error_message());
        std::process::exit(3);
    }
    eprintln!("running {} frames", n_frames);
    for f in 0..n_frames {
        // Simulate pressing X (buttons[5]=0x20) for the first 8 frames to advance past title
        if f < 8 {
            state.input.btn_state[0] = 0x20;
        } else {
            state.input.btn_state[0] = 0;
        }
        state.input.update();
        state.memory.ram[0x5F4C] = state.input.btn_state[0];
        lua.call_update(&mut state);
        if lua.had_error() {
            eprintln!("UPDATE ERROR (frame {}): {}", f, lua.error_message());
            std::process::exit(4);
        }
        lua.call_draw(&mut state);
        if lua.had_error() {
            eprintln!("DRAW ERROR (frame {}): {}", f, lua.error_message());
            std::process::exit(5);
        }
        state.frame_count += 1;
        state.target_fps = if lua.use_60fps() { 60 } else { 30 };
        state.elapsed_time += 1.0 / state.target_fps as f64;
    }
    eprintln!("ok, ran {} frames clean", n_frames);
    // Sample the screen — count nonzero pixels in last frame
    let screen = &state.memory.ram[0x6000..0x8000];
    let nonzero = screen.iter().filter(|&&b| b != 0).count();
    eprintln!("screen: {} / 8192 bytes nonzero", nonzero);
}
