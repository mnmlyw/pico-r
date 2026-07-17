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
pub const ADDR_LINE_X: u16 = 0x5F3C; // last line() endpoint, 16-bit
pub const ADDR_LINE_Y: u16 = 0x5F3E;

pub const ADDR_INPUT_P0: u16 = 0x5F4C;
pub const ADDR_INPUT_P1: u16 = 0x5F4D;

// Sprite/screen page registers (high byte of base address)
pub const ADDR_SPRITE_PAGE: u16 = 0x5F54; // default 0x00 → sprite sheet at 0x0000
pub const ADDR_SCREEN_PAGE: u16 = 0x5F55; // default 0x60 → screen at 0x6000

pub struct Memory {
    pub ram: [u8; RAM_SIZE],
    pub rom: [u8; RAM_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub fn new() -> Self {
        let mut m = Self {
            ram: [0; RAM_SIZE],
            rom: [0; RAM_SIZE],
        };
        // Page registers must default to PICO-8's expected base addresses,
        // not zero — otherwise sprite/screen ops index from address 0 instead
        // of 0x6000.
        m.ram[ADDR_SPRITE_PAGE as usize] = 0x00;
        m.ram[ADDR_SCREEN_PAGE as usize] = 0x60;
        m
    }

    /// High-byte page registers — sprite/draw operations consult these to find
    /// the actual base address. Carts can poke 0x5F54/0x5F55 to redirect
    /// sprite-read or screen-write to a different region of RAM (e.g.
    /// bigprint's "draw small text, then sspr-scale it from screen-as-sprite"
    /// trick that pokes 0x5F54=0x60).
    #[inline]
    pub fn sprite_base(&self) -> u16 {
        (self.ram[ADDR_SPRITE_PAGE as usize] as u16) << 8
    }

    #[inline]
    pub fn screen_base(&self) -> u16 {
        (self.ram[ADDR_SCREEN_PAGE as usize] as u16) << 8
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
        // Sprite/screen page defaults: sprite reads from 0x0000, draws go to 0x6000.
        self.ram[ADDR_SPRITE_PAGE as usize] = 0x00;
        self.ram[ADDR_SCREEN_PAGE as usize] = 0x60;
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
    /// Honors the 0x5F55 screen-page register so carts that redirect draws
    /// (e.g. poke(0x5F55, 0x00) to draw to the sprite sheet) work.
    #[inline]
    pub fn screen_get(&self, x: u8, y: u8) -> u8 {
        let addr = (self.screen_base() as usize)
            .wrapping_add((y as usize) * 64)
            .wrapping_add((x as usize) / 2)
            & 0xFFFF;
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
        let addr = (self.screen_base() as usize)
            .wrapping_add((y as usize) * 64)
            .wrapping_add((x as usize) / 2)
            & 0xFFFF;
        if x & 1 == 0 {
            self.ram[addr] = (self.ram[addr] & 0xF0) | (color & 0x0F);
        } else {
            self.ram[addr] = (self.ram[addr] & 0x0F) | ((color & 0x0F) << 4);
        }
    }

    /// Sprite-sheet pixel access. Honors the 0x5F54 sprite-page register so
    /// spr/sspr/map can read pixel data from a redirected base (e.g. bigprint
    /// pokes 0x5F54=0x60 to read sprites from the screen, scaling rendered text).
    #[inline]
    pub fn sprite_get(&self, x: u8, y: u8) -> u8 {
        if x >= 128 || y >= 128 {
            return 0;
        }
        let addr = (self.sprite_base() as usize)
            .wrapping_add((y as usize) * 64)
            .wrapping_add((x as usize) / 2)
            & 0xFFFF;
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
        let addr = (self.sprite_base() as usize)
            .wrapping_add((y as usize) * 64)
            .wrapping_add((x as usize) / 2)
            & 0xFFFF;
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
