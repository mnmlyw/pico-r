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
    }
}
