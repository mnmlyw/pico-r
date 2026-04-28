pub const RAM_SIZE: usize = 65536;

// PICO-8 memory map
pub const ADDR_SPRITE: u16 = 0x0000;
pub const ADDR_SHARED: u16 = 0x1000;
pub const ADDR_MAP: u16 = 0x2000;
pub const ADDR_FLAGS: u16 = 0x3000;
pub const ADDR_MUSIC: u16 = 0x3100;
pub const ADDR_SFX: u16 = 0x3200;
pub const ADDR_GENERAL: u16 = 0x4300;
pub const ADDR_CART_DATA: u16 = 0x5E00;
pub const ADDR_DRAW_STATE: u16 = 0x5F00;
pub const ADDR_HW_STATE: u16 = 0x5F40;
pub const ADDR_GPIO: u16 = 0x5F80;
pub const ADDR_SCREEN: u16 = 0x6000;
pub const ADDR_SCREEN_END: u32 = 0x8000;

pub const ADDR_DRAW_PAL: u16 = 0x5F00;
pub const ADDR_SCREEN_PAL: u16 = 0x5F10;
pub const ADDR_CLIP_LEFT: u16 = 0x5F20;
pub const ADDR_CLIP_TOP: u16 = 0x5F21;
pub const ADDR_CLIP_RIGHT: u16 = 0x5F22;
pub const ADDR_CLIP_BOTTOM: u16 = 0x5F23;
pub const ADDR_COLOR: u16 = 0x5F25;
pub const ADDR_CURSOR_X: u16 = 0x5F26;
pub const ADDR_CURSOR_Y: u16 = 0x5F27;
pub const ADDR_CAMERA_X: u16 = 0x5F28;
pub const ADDR_CAMERA_Y: u16 = 0x5F2A;
pub const ADDR_DEVKIT: u16 = 0x5F2D;
pub const ADDR_FILL_PAT: u16 = 0x5F31;

pub const ADDR_INPUT_P0: u16 = 0x5F4C;
pub const ADDR_INPUT_P1: u16 = 0x5F4D;

pub struct Memory {
    pub ram: [u8; RAM_SIZE],
    pub rom: [u8; RAM_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            ram: [0; RAM_SIZE],
            rom: [0; RAM_SIZE],
        }
    }

    pub fn init_draw_state(&mut self) {
        for i in 0..16 {
            self.ram[ADDR_DRAW_PAL as usize + i] = i as u8;
            self.ram[ADDR_SCREEN_PAL as usize + i] = i as u8;
        }
        self.ram[ADDR_CLIP_LEFT as usize] = 0;
        self.ram[ADDR_CLIP_TOP as usize] = 0;
        self.ram[ADDR_CLIP_RIGHT as usize] = 128;
        self.ram[ADDR_CLIP_BOTTOM as usize] = 128;
        self.ram[ADDR_COLOR as usize] = 6;
        self.poke16(ADDR_CAMERA_X, 0);
        self.poke16(ADDR_CAMERA_Y, 0);
        self.ram[ADDR_CURSOR_X as usize] = 0;
        self.ram[ADDR_CURSOR_Y as usize] = 0;
        self.poke16(ADDR_FILL_PAT, 0);
        self.poke16(ADDR_FILL_PAT.wrapping_add(2), 0);
        self.ram[ADDR_DRAW_PAL as usize] |= 0x10;
    }

    pub fn save_rom(&mut self) {
        self.rom.copy_from_slice(&self.ram);
    }

    #[inline]
    pub fn peek(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    #[inline]
    pub fn poke(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }

    #[inline]
    pub fn peek16(&self, addr: u16) -> u16 {
        let a = self.ram[addr as usize] as u16;
        let b = self.ram[addr.wrapping_add(1) as usize] as u16;
        a | (b << 8)
    }

    #[inline]
    pub fn poke16(&mut self, addr: u16, val: u16) {
        self.ram[addr as usize] = val as u8;
        self.ram[addr.wrapping_add(1) as usize] = (val >> 8) as u8;
    }

    #[inline]
    pub fn peek32(&self, addr: u16) -> u32 {
        let a = self.ram[addr as usize] as u32;
        let b = self.ram[addr.wrapping_add(1) as usize] as u32;
        let c = self.ram[addr.wrapping_add(2) as usize] as u32;
        let d = self.ram[addr.wrapping_add(3) as usize] as u32;
        a | (b << 8) | (c << 16) | (d << 24)
    }

    #[inline]
    pub fn poke32(&mut self, addr: u16, val: u32) {
        self.ram[addr as usize] = val as u8;
        self.ram[addr.wrapping_add(1) as usize] = (val >> 8) as u8;
        self.ram[addr.wrapping_add(2) as usize] = (val >> 16) as u8;
        self.ram[addr.wrapping_add(3) as usize] = (val >> 24) as u8;
    }

    /// Read a 4-bit pixel from the screen at (x, y). x,y in [0,127].
    #[inline]
    pub fn screen_get(&self, x: u8, y: u8) -> u8 {
        let addr = ADDR_SCREEN as usize + (y as usize) * 64 + (x as usize) / 2;
        let byte = self.ram[addr];
        if x & 1 == 0 {
            byte & 0x0F
        } else {
            byte >> 4
        }
    }

    #[inline]
    pub fn screen_set(&mut self, x: u8, y: u8, color: u8) {
        let x = x & 0x7F;
        let y = y & 0x7F;
        let addr = ADDR_SCREEN as usize + (y as usize) * 64 + (x as usize) / 2;
        if x & 1 == 0 {
            self.ram[addr] = (self.ram[addr] & 0xF0) | (color & 0x0F);
        } else {
            self.ram[addr] = (self.ram[addr] & 0x0F) | ((color & 0x0F) << 4);
        }
    }

    #[inline]
    pub fn sprite_get(&self, x: u8, y: u8) -> u8 {
        if x >= 128 || y >= 128 {
            return 0;
        }
        let addr = ADDR_SPRITE as usize + (y as usize) * 64 + (x as usize) / 2;
        let byte = self.ram[addr];
        if x & 1 == 0 {
            byte & 0x0F
        } else {
            byte >> 4
        }
    }

    #[inline]
    pub fn sprite_set(&mut self, x: u8, y: u8, color: u8) {
        if x >= 128 || y >= 128 {
            return;
        }
        let addr = ADDR_SPRITE as usize + (y as usize) * 64 + (x as usize) / 2;
        if x & 1 == 0 {
            self.ram[addr] = (self.ram[addr] & 0xF0) | (color & 0x0F);
        } else {
            self.ram[addr] = (self.ram[addr] & 0x0F) | ((color & 0x0F) << 4);
        }
    }

    pub fn map_get(&self, x: u8, y: u8) -> u8 {
        if x >= 128 || y >= 64 {
            return 0;
        }
        if y < 32 {
            self.ram[ADDR_MAP as usize + (y as usize) * 128 + x as usize]
        } else {
            self.ram[ADDR_SHARED as usize + ((y - 32) as usize) * 128 + x as usize]
        }
    }

    pub fn map_set(&mut self, x: u8, y: u8, val: u8) {
        if x >= 128 || y >= 64 {
            return;
        }
        if y < 32 {
            self.ram[ADDR_MAP as usize + (y as usize) * 128 + x as usize] = val;
        } else {
            self.ram[ADDR_SHARED as usize + ((y - 32) as usize) * 128 + x as usize] = val;
        }
    }

    pub fn memcpy(&mut self, dst: u16, src: u16, len: u16) {
        if len == 0 {
            return;
        }
        if dst < src {
            for i in 0..len {
                let s = src.wrapping_add(i) as usize;
                let d = dst.wrapping_add(i) as usize;
                self.ram[d] = self.ram[s];
            }
        } else {
            for i in (0..len).rev() {
                let s = src.wrapping_add(i) as usize;
                let d = dst.wrapping_add(i) as usize;
                self.ram[d] = self.ram[s];
            }
        }
    }

    pub fn memset(&mut self, dst: u16, val: u8, len: u16) {
        for i in 0..len {
            self.ram[dst.wrapping_add(i) as usize] = val;
        }
    }

    pub fn reload(&mut self, dst: u16, src: u16, len: u16) {
        for i in 0..len {
            self.ram[dst.wrapping_add(i) as usize] = self.rom[src.wrapping_add(i) as usize];
        }
    }
}
