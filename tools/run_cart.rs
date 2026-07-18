// Host-side cart runner — loads a cart, executes a few frames, prints errors.
// Usage: cargo run --bin run-cart -- path/to/cart.p8.png [n_frames]

use pico_r::{cart, lua_engine::LuaEngine, pico_lua::LuaImpl, preprocessor, state::PicoState};

fn main() {
    // The tree-walking interpreter spends native stack per Lua call
    // level; deeply recursive carts (embedded compilers/VMs) need far
    // more than the default main-thread stack. Run everything on a
    // big-stack worker so the Lua recursion cap can sit high.
    let child = std::thread::Builder::new()
        .stack_size(1024 * 1024 * 1024)
        .spawn(run_main)
        .expect("spawn worker");
    match child.join() {
        Ok(()) => {}
        Err(_) => std::process::exit(134),
    }
}

fn run_main() {
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
    // Old-style carts run their whole game as an explicit `flip()`+goto
    // main loop at top level instead of defining _update/_draw -- give
    // them a frame budget so they count as "ran N frames" instead of
    // hanging a headless run forever. Generous slack over n_frames so
    // legitimate init-time flips (loading screens) never trip it.
    state.flip_limit = n_frames + 60;

    eprintln!("cart loaded; lua source {} bytes", cart.lua_code.len());
    let processed = preprocessor::preprocess(&cart.lua_code);
    eprintln!("preprocessed {} bytes", processed.len());

    let flip_limit_hit = |msg: &str| msg.contains(pico_r::pico_lua::api::FLIP_LIMIT_MARKER);

    let mut lua = LuaImpl::new();
    lua.set_recursion_limit(150_000);
    if let Err(e) = lua.load_cart(&mut state, &cart) {
        if flip_limit_hit(&e) {
            eprintln!("ok, explicit-flip main loop ran {} frames clean", n_frames);
            report_and_exit(&mut state);
        }
        eprintln!("LOAD ERROR: {}", e);
        // Dump the relevant part of preprocessed source for context
        eprintln!("---- preprocessed (head) ----");
        for line in processed.split(|&b| b == b'\n').take(20) {
            eprintln!("{}", String::from_utf8_lossy(line));
        }
        std::process::exit(2);
    }
    eprintln!("loaded; calling _init");
    lua.call_init(&mut state);
    if lua.had_error() {
        let msg = lua.error_message();
        if flip_limit_hit(msg) {
            eprintln!("ok, explicit-flip main loop ran {} frames clean", n_frames);
            report_and_exit(&mut state);
        }
        eprintln!("INIT ERROR: {}", msg);
        std::process::exit(3);
    }
    eprintln!("running {} frames", n_frames);
    for f in 0..n_frames {
        // Simulate button presses to advance past title screens: X
        // (buttons[5]=0x20) for the first 8 frames, then O
        // (buttons[4]=0x10) for the next 8 -- carts vary in which button
        // their "press to start" waits for.
        if f < 8 {
            state.input.btn_state[0] = 0x20;
        } else if f < 16 {
            state.input.btn_state[0] = 0x10;
        } else {
            state.input.btn_state[0] = 0;
        }
        state.input.update();
        state.memory.ram[0x5F4C] = state.input.btn_state[0];
        lua.call_update(&mut state);
        if lua.had_error() {
            let msg = lua.error_message();
            if flip_limit_hit(msg) {
                eprintln!("ok, explicit-flip main loop ran {} frames clean", n_frames);
                report_and_exit(&mut state);
            }
            eprintln!("UPDATE ERROR (frame {}): {}", f, msg);
            std::process::exit(4);
        }
        lua.call_draw(&mut state);
        if lua.had_error() {
            let msg = lua.error_message();
            if flip_limit_hit(msg) {
                eprintln!("ok, explicit-flip main loop ran {} frames clean", n_frames);
                report_and_exit(&mut state);
            }
            eprintln!("DRAW ERROR (frame {}): {}", f, msg);
            std::process::exit(5);
        }
        state.frame_count += 1;
        state.target_fps = if lua.use_60fps() { 60 } else { 30 };
        state.elapsed_time += 1.0 / state.target_fps as f64;
    }
    eprintln!("ok, ran {} frames clean", n_frames);
    report_and_exit(&mut state);
}

/// Print the screen/audio summary and exit 0 (the "cart ran clean" path).
fn report_and_exit(state: &mut PicoState) -> ! {
    // Sample the screen — count nonzero pixels in last frame
    let screen = &state.memory.ram[0x6000..0x8000];
    let nonzero = screen.iter().filter(|&&b| b != 0).count();
    eprintln!("screen: {} / 8192 bytes nonzero", nonzero);

    // Sanity-check audio engine: pull 22050 samples (1 second) and report
    // peak amplitude + nonzero count. Catches "engine never produces audio".
    let mut peak: f32 = 0.0;
    let mut nonzero_samples = 0usize;
    for _ in 0..22050 {
        let s = state.audio.generate_sample(&state.memory);
        if s.abs() > peak {
            peak = s.abs();
        }
        if s != 0.0 {
            nonzero_samples += 1;
        }
    }
    eprintln!(
        "audio: peak={:.4} nonzero={}/22050 music_pattern={} channels=[{}]",
        peak,
        nonzero_samples,
        state.audio.music_state.pattern,
        state
            .audio
            .channels
            .iter()
            .map(|c| format!("sfx={} fin={} vol={:.2}", c.sfx_id, c.finished, c.volume))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    std::process::exit(0);
}
