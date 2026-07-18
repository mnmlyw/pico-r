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
    // Real PICO-8's rnd()/srand() PRNG state (oracle-confirmed algorithm,
    // see api.rs's rng_step/api_srand): two interleaved 32-bit words seeded
    // via srand()'s xor+32-warmup. The default here is that same derivation
    // applied to seed 0 -- there's no "correct" default to match, since
    // official PICO-8 itself seeds an un-srand()'d cart from wall-clock time
    // (confirmed non-reproducible run-to-run), so any fixed default is only
    // ever an approximation for the un-seeded case.
    pub rng_hi: u32,
    pub rng_lo: u32,

    // line() pen state
    pub line_x: i32,
    pub line_y: i32,
    pub line_valid: bool,
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
            rng_hi: 0xd67ce1e8,
            rng_lo: 0x42cfadf8,
            line_x: 0,
            line_y: 0,
            line_valid: false,
        }
    }

    pub fn prepare_for_cart_load(&mut self) {
        self.audio.reset();
        self.rng_hi = 0xd67ce1e8;
        self.rng_lo = 0x42cfadf8;
        self.elapsed_time = 0.0;
        self.frame_count = 0;
        self.line_x = 0;
        self.line_y = 0;
        self.line_valid = false;
    }
}
