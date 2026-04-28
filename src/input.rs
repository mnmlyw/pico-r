use crate::memory::Memory;

pub struct Input {
    pub btn_state: [u8; 2],
    pub prev_state: [u8; 2],
    pub held_frames: [[u16; 8]; 2],
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_buttons: u8,
    pub mouse_wheel: i32,
    pub key_chars: [u8; 32],
    pub key_chars_len: u8,
}

impl Input {
    pub fn new() -> Self {
        Self {
            btn_state: [0; 2],
            prev_state: [0; 2],
            held_frames: [[0; 8]; 2],
            mouse_x: 0,
            mouse_y: 0,
            mouse_buttons: 0,
            mouse_wheel: 0,
            key_chars: [0; 32],
            key_chars_len: 0,
        }
    }

    /// Update held-frame counters from current btn_state.
    /// btn_state itself is set externally (by JS via WASM exports).
    pub fn update(&mut self) {
        self.prev_state = self.btn_state;
        for p in 0..2 {
            for b in 0..8 {
                let mask: u8 = 1 << b;
                if self.btn_state[p] & mask != 0 {
                    self.held_frames[p][b] = self.held_frames[p][b].saturating_add(1);
                } else {
                    self.held_frames[p][b] = 0;
                }
            }
        }
    }

    pub fn btn(&self, button: u8, player: u8) -> bool {
        let p = (player & 1) as usize;
        let b = button & 7;
        self.btn_state[p] & (1u8 << b) != 0
    }

    pub fn btnp(&self, button: u8, player: u8, memory: &Memory) -> bool {
        let p = (player & 1) as usize;
        let b = (button & 7) as usize;
        let held = self.held_frames[p][b];
        if held == 0 {
            return false;
        }
        if held == 1 {
            return true;
        }
        let initial_raw = memory.ram[0x5F5C];
        let repeat_raw = memory.ram[0x5F5D];
        let initial: u16 = if initial_raw == 0 { 15 } else { initial_raw as u16 };
        if initial == 255 {
            return false;
        }
        let repeat: u16 = if repeat_raw == 0 { 4 } else { repeat_raw as u16 };
        if held >= initial && (held - initial) % repeat == 0 {
            return true;
        }
        false
    }
}
