use crate::audio::Audio;
use crate::input::Input;
use crate::memory::Memory;

pub const SCREEN_W: usize = 128;
pub const SCREEN_H: usize = 128;

pub struct PicoState {
    pub memory: Memory,
    pub input: Input,
    pub audio: Audio,
    pub pixel_buffer: [u32; SCREEN_W * SCREEN_H],

    pub frame_count: u32,
    pub elapsed_time: f64,
    pub target_fps: u8,
    pub rng_state: u32,

    // line() pen state
    pub line_x: i32,
    pub line_y: i32,
    pub line_valid: bool,

    /// Number of times the cart has called flip() explicitly. Old-style
    /// carts run their whole game as `::_:: ... flip() goto _` at top
    /// level instead of defining _update/_draw.
    pub flip_count: u32,
    /// When nonzero, a flip() call past this count raises a special
    /// interpreter error (see api_flip) so a headless host can treat an
    /// explicit-flip main loop as "ran N frames" instead of hanging
    /// forever. 0 = unlimited (interactive/WASM hosts).
    pub flip_limit: u32,
    /// btn()/btnp() calls since the last frame tick -- input-wait busy
    /// loops (`while(not btn(4))do ... end`) poll without ever flipping;
    /// a headless host counts sustained polling as synthetic frames so
    /// the flip budget still ends the loop.
    pub btn_poll_count: u32,
    /// Bytes queued via serial() -- stat(108) reports this (confirmed via
    /// oracle: each serial(0x808,addr,len) call adds len).
    pub serial_queued: u32,
}

impl Default for PicoState {
    fn default() -> Self {
        Self::new()
    }
}

impl PicoState {
    pub fn new() -> Self {
        let mut memory = Memory::new();
        memory.init_draw_state();
        Self {
            memory,
            input: Input::new(),
            audio: Audio::new(),
            pixel_buffer: [0xFF000000; SCREEN_W * SCREEN_H],
            frame_count: 0,
            elapsed_time: 0.0,
            target_fps: 30,
            rng_state: 1,
            line_x: 0,
            line_y: 0,
            line_valid: false,
            flip_count: 0,
            flip_limit: 0,
            btn_poll_count: 0,
            serial_queued: 0,
        }
    }

    pub fn prepare_for_cart_load(&mut self) {
        self.audio.reset();
        self.rng_state = 1;
        self.elapsed_time = 0.0;
        self.frame_count = 0;
        self.line_x = 0;
        self.line_y = 0;
        self.line_valid = false;
        self.flip_count = 0;
        self.btn_poll_count = 0;
        self.serial_queued = 0;
    }
}
