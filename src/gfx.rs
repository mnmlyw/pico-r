use crate::gfx_font;
use crate::memory::{self, Memory};
use crate::palette;
use crate::state::PicoState;

pub const SCREEN_W: usize = 128;
pub const SCREEN_H: usize = 128;

pub fn get_camera(memory: &Memory) -> (i32, i32) {
    let cx = memory.peek16(memory::ADDR_CAMERA_X) as i16 as i32;
    let cy = memory.peek16(memory::ADDR_CAMERA_Y) as i16 as i32;
    (cx, cy)
}

pub fn get_clip(memory: &Memory) -> (i32, i32, i32, i32) {
    (
        memory.ram[memory::ADDR_CLIP_LEFT as usize] as i32,
        memory.ram[memory::ADDR_CLIP_TOP as usize] as i32,
        memory.ram[memory::ADDR_CLIP_RIGHT as usize] as i32,
        memory.ram[memory::ADDR_CLIP_BOTTOM as usize] as i32,
    )
}

#[inline]
fn get_draw_pal(memory: &Memory, col: u8) -> u8 {
    memory.ram[memory::ADDR_DRAW_PAL as usize + (col & 0x0F) as usize] & 0x0F
}

#[inline]
pub fn is_transparent(memory: &Memory, col: u8) -> bool {
    memory.ram[memory::ADDR_DRAW_PAL as usize + (col & 0x0F) as usize] & 0x10 != 0
}

#[inline]
fn get_screen_pal(memory: &Memory, col: u8) -> u8 {
    memory.ram[memory::ADDR_SCREEN_PAL as usize + (col & 0x0F) as usize] & 0x0F
}

#[inline]
fn get_fill_pattern(memory: &Memory) -> u16 {
    memory.peek16(memory::ADDR_FILL_PAT)
}

pub fn put_pixel(memory: &mut Memory, x: i32, y: i32, col: u8) {
    let (cx, cy) = get_camera(memory);
    put_pixel_raw(memory, x - cx, y - cy, col);
}

pub fn put_pixel_raw(memory: &mut Memory, sx: i32, sy: i32, col: u8) {
    let (x0, y0, x1, y1) = get_clip(memory);
    if sx < x0 || sx >= x1 || sy < y0 || sy >= y1 {
        return;
    }
    if !(0..128).contains(&sx) || !(0..128).contains(&sy) {
        return;
    }
    let pat = get_fill_pattern(memory);
    if pat != 0 {
        let px = sx as u32 & 3;
        let py = sy as u32 & 3;
        let bit_idx = py * 4 + px;
        if pat & (1 << bit_idx) != 0 {
            let fill_trans = memory.ram[memory::ADDR_FILL_PAT as usize + 2];
            if fill_trans & 0x1 != 0 {
                return;
            }
            let color_byte = memory.ram[memory::ADDR_COLOR as usize];
            let secondary = color_byte >> 4;
            let mapped = get_draw_pal(memory, secondary);
            memory.screen_set((sx as u32 & 0x7F) as u8, (sy as u32 & 0x7F) as u8, mapped);
            return;
        }
    }
    let mapped = get_draw_pal(memory, col);
    memory.screen_set((sx as u32 & 0x7F) as u8, (sy as u32 & 0x7F) as u8, mapped);
}

pub fn put_pixel_no_cam(memory: &mut Memory, sx: i32, sy: i32, col: u8) {
    let (x0, y0, x1, y1) = get_clip(memory);
    if sx < x0 || sx >= x1 || sy < y0 || sy >= y1 {
        return;
    }
    if !(0..128).contains(&sx) || !(0..128).contains(&sy) {
        return;
    }
    let mapped = get_draw_pal(memory, col);
    memory.screen_set((sx as u32 & 0x7F) as u8, (sy as u32 & 0x7F) as u8, mapped);
}

pub fn render_to_argb(memory: &Memory, pixel_buffer: &mut [u32; SCREEN_W * SCREEN_H]) {
    let mode = memory.ram[0x5F2C];
    for y in 0..128usize {
        for x in 0..128usize {
            let (sx, sy) = match mode {
                1 => (x / 2, y),
                2 => (x, y / 2),
                3 => (x / 2, y / 2),
                5 => (127 - x, y),
                6 => (x, 127 - y),
                7 => (127 - x, 127 - y),
                _ => (x, y),
            };
            let col = memory.screen_get(sx as u8, sy as u8);
            let screen_col = get_screen_pal(memory, col);
            pixel_buffer[y * 128 + x] = palette::COLORS[screen_col as usize];
        }
    }
}

// === Drawing primitives ===

pub fn cls(state: &mut PicoState, col: u8) {
    let c = col & 0x0F;
    let byte = c | (c << 4);
    // Honor 0x5F55 in case the cart has redirected the screen base.
    let start = state.memory.screen_base() as usize;
    let end = (start + 0x2000).min(memory::RAM_SIZE);
    for i in start..end {
        state.memory.ram[i] = byte;
    }
    state.memory.ram[memory::ADDR_CURSOR_X as usize] = 0;
    state.memory.ram[memory::ADDR_CURSOR_Y as usize] = 0;
    state.memory.ram[memory::ADDR_CLIP_LEFT as usize] = 0;
    state.memory.ram[memory::ADDR_CLIP_TOP as usize] = 0;
    state.memory.ram[memory::ADDR_CLIP_RIGHT as usize] = 128;
    state.memory.ram[memory::ADDR_CLIP_BOTTOM as usize] = 128;
}

pub fn draw_line(memory: &mut Memory, x0: i32, y0: i32, x1: i32, y1: i32, col: u8) {
    let dx = if x1 > x0 { x1 - x0 } else { x0 - x1 };
    let dy = if y1 > y0 { y1 - y0 } else { y0 - y1 };
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x0;
    let mut cy = y0;
    loop {
        put_pixel(memory, cx, cy, col);
        if cx == x1 && cy == y1 {
            break;
        }
        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
    }
}

pub fn rect(memory: &mut Memory, x0: i32, y0: i32, x1: i32, y1: i32, col: u8) {
    draw_line(memory, x0, y0, x1, y0, col);
    draw_line(memory, x1, y0, x1, y1, col);
    draw_line(memory, x1, y1, x0, y1, col);
    draw_line(memory, x0, y1, x0, y0, col);
}

pub fn rectfill(memory: &mut Memory, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, col: u8) {
    if x0 > x1 {
        core::mem::swap(&mut x0, &mut x1);
    }
    if y0 > y1 {
        core::mem::swap(&mut y0, &mut y1);
    }
    let mut y = y0;
    while y <= y1 {
        let mut x = x0;
        while x <= x1 {
            put_pixel(memory, x, y, col);
            x += 1;
        }
        y += 1;
    }
}

pub fn circ(memory: &mut Memory, cx: i32, cy: i32, r: i32, col: u8) {
    draw_circ(memory, cx, cy, r, col, false);
}

pub fn circfill(memory: &mut Memory, cx: i32, cy: i32, r: i32, col: u8) {
    draw_circ(memory, cx, cy, r, col, true);
}

fn draw_circ(memory: &mut Memory, cx: i32, cy: i32, r: i32, col: u8, fill: bool) {
    if r < 0 {
        return;
    }
    let (cam_x, cam_y) = get_camera(memory);
    let acx = cx - cam_x;
    let acy = cy - cam_y;
    let invert = fill && (memory.ram[0x5F34] & 0x2 != 0);

    if r == 0 {
        if invert {
            invert_fill_circ(memory, acx, acy, 0, col);
        } else {
            put_pixel_raw(memory, acx, acy, col);
        }
        return;
    }
    if invert {
        invert_fill_circ(memory, acx, acy, r, col);
        return;
    }
    let mut x: i32 = r;
    let mut y: i32 = 0;
    let mut d: i32 = 1 - r;
    while x >= y {
        if fill {
            hline(memory, acx - x, acx + x, acy + y, col);
            hline(memory, acx - x, acx + x, acy - y, col);
            hline(memory, acx - y, acx + y, acy + x, col);
            hline(memory, acx - y, acx + y, acy - x, col);
        } else {
            put_pixel_raw(memory, acx + x, acy + y, col);
            put_pixel_raw(memory, acx - x, acy + y, col);
            put_pixel_raw(memory, acx + x, acy - y, col);
            put_pixel_raw(memory, acx - x, acy - y, col);
            put_pixel_raw(memory, acx + y, acy + x, col);
            put_pixel_raw(memory, acx - y, acy + x, col);
            put_pixel_raw(memory, acx + y, acy - x, col);
            put_pixel_raw(memory, acx - y, acy - x, col);
        }
        y += 1;
        if d < 0 {
            d += 2 * y + 1;
        } else {
            x -= 1;
            d += 2 * (y - x) + 1;
        }
    }
}

fn hline(memory: &mut Memory, x0: i32, x1: i32, y: i32, col: u8) {
    let mut sx = x0;
    while sx <= x1 {
        put_pixel_raw(memory, sx, y, col);
        sx += 1;
    }
}

fn invert_fill_circ(memory: &mut Memory, cx: i32, cy: i32, r: i32, col: u8) {
    let (x0, y0, x1, y1) = get_clip(memory);
    let mut sy = y0;
    while sy < y1 {
        let dy = sy - cy;
        if dy < -r || dy > r {
            hline(memory, x0, x1 - 1, sy, col);
        } else {
            let dy2 = dy * dy;
            let r2 = r * r;
            let dx = ((r2 - dy2) as f64).sqrt() as i32;
            let left = cx - dx;
            let right = cx + dx;
            if x0 < left {
                hline(memory, x0, left - 1, sy, col);
            }
            if right + 1 < x1 {
                hline(memory, right + 1, x1 - 1, sy, col);
            }
        }
        sy += 1;
    }
}

pub fn oval(memory: &mut Memory, x0: i32, y0: i32, x1: i32, y1: i32, col: u8) {
    draw_oval(memory, x0, y0, x1, y1, col, false);
}

pub fn ovalfill(memory: &mut Memory, x0: i32, y0: i32, x1: i32, y1: i32, col: u8) {
    draw_oval(memory, x0, y0, x1, y1, col, true);
}

fn draw_oval(memory: &mut Memory, x0: i32, y0: i32, x1: i32, y1: i32, col: u8, fill: bool) {
    // Per-row exact scan (not an incremental midpoint stepper): the
    // previous single-loop midpoint-ellipse implementation could
    // terminate before x reached 0, missing the top/bottom pole pixels
    // entirely (confirmed against official PICO-8 even for an
    // exact-integer bounding box, not just an odd-sized one). Scanning
    // every row directly from the ellipse equation can't have that class
    // of bug, and correctly handles half-integer centers/radii (odd
    // bounding boxes) without truncating them first.
    let (x0, x1) = (x0.min(x1), x0.max(x1));
    let (y0, y1) = (y0.min(y1), y0.max(y1));
    // x0..=x1 is an INCLUSIVE pixel range, so in continuous coordinates
    // (each pixel `p` spans [p, p+1)) the box spans x0..(x1+1) -- confirmed
    // row-by-row against official PICO-8: using (x1-x0)/2 as the radius
    // (excluding the +1) shifts the whole curve and breaks top/bottom
    // symmetry (rows above center came out too wide, rows below too
    // narrow, and the bottom pole row vanished entirely).
    let cx = (x0 + x1 + 1) as f64 / 2.0;
    let cy = (y0 + y1 + 1) as f64 / 2.0;
    let rx = (x1 - x0 + 1) as f64 / 2.0;
    let ry = (y1 - y0 + 1) as f64 / 2.0;
    if x0 == x1 && y0 == y1 {
        put_pixel(memory, x0, y0, col);
        return;
    }
    for py in y0..=y1 {
        // Sample at the pixel's vertical center.
        let dy = (py as f64 + 0.5) - cy;
        let t = 1.0 - (dy / ry) * (dy / ry);
        if t < 0.0 {
            continue;
        }
        let dx_extent = rx * t.sqrt();
        // A column's CENTER (px+0.5) must fall within [cx-dx_extent,
        // cx+dx_extent] for that column to be part of the shape.
        let lx = (cx - dx_extent - 0.5).ceil() as i32;
        let rx2 = (cx + dx_extent - 0.5).floor() as i32;
        if fill {
            // Not hline(): it calls put_pixel_raw directly and skips the
            // camera-offset transform that put_pixel applies.
            let mut px = lx;
            while px <= rx2 {
                put_pixel(memory, px, py, col);
                px += 1;
            }
        } else {
            put_pixel(memory, lx, py, col);
            put_pixel(memory, rx2, py, col);
        }
    }
    if !fill {
        // A row-only scan leaves gaps near the top/bottom poles: official
        // PICO-8 draws a flat horizontal run of several pixels there
        // (confirmed -- e.g. the very top row of a wide, short oval has
        // ~11 pixels lit, not just its 2 boundary columns), because the
        // curve is nearly horizontal in that region. A column scan (top
        // and bottom per column) naturally fills exactly that gap, the
        // same way the row scan handles the near-vertical left/right
        // regions -- together they give a complete, gap-free outline.
        for px in x0..=x1 {
            let dx = (px as f64 + 0.5) - cx;
            let t = 1.0 - (dx / rx) * (dx / rx);
            if t < 0.0 {
                continue;
            }
            let dy_extent = ry * t.sqrt();
            let ty = (cy - dy_extent - 0.5).ceil() as i32;
            let by = (cy + dy_extent - 0.5).floor() as i32;
            put_pixel(memory, px, ty, col);
            put_pixel(memory, px, by, col);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spr(
    memory: &mut Memory,
    n: i32,
    x: i32,
    y: i32,
    w: f64,
    h: f64,
    flip_x: bool,
    flip_y: bool,
) {
    let pw = (w * 8.0) as i32;
    let ph = (h * 8.0) as i32;
    let sx = (n.rem_euclid(16)) * 8;
    let sy = (n.div_euclid(16)) * 8;
    draw_sprite(memory, sx, sy, x, y, pw, ph, flip_x, flip_y);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_sprite(
    memory: &mut Memory,
    sx: i32,
    sy: i32,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    flip_x: bool,
    flip_y: bool,
) {
    let (cam_x, cam_y) = get_camera(memory);
    for py in 0..h {
        for px in 0..w {
            let src_x = sx + if flip_x { w - 1 - px } else { px };
            let src_y = sy + if flip_y { h - 1 - py } else { py };
            if !(0..128).contains(&src_x) || !(0..128).contains(&src_y) {
                continue;
            }
            let col = memory.sprite_get(src_x as u8, src_y as u8);
            if is_transparent(memory, col) {
                continue;
            }
            put_pixel_raw(memory, dx + px - cam_x, dy + py - cam_y, col);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sspr(
    memory: &mut Memory,
    sx: i32,
    sy: i32,
    sw: i32,
    sh: i32,
    dx: i32,
    dy: i32,
    dw: i32,
    dh: i32,
    flip_x: bool,
    flip_y: bool,
) {
    if dw <= 0 || dh <= 0 || sw <= 0 || sh <= 0 {
        return;
    }
    if sw > 256 || sh > 256 {
        return;
    }
    let (cam_x, cam_y) = get_camera(memory);

    // Snapshot the source rectangle first. When sprite source and screen
    // destination overlap (e.g. bigprint's "scale up the just-drawn text"
    // trick that pokes 0x5F54=0x60), iterating in place would overwrite
    // source pixels before we've read them. PICO-8 reads atomically; mimic.
    let src_w = sw as usize;
    let src_h = sh as usize;
    let mut src_buf: Vec<u8> = vec![0u8; src_w * src_h];
    for sj in 0..sh {
        for si in 0..sw {
            let ax = sx + si;
            let ay = sy + sj;
            let idx = (sj as usize) * src_w + (si as usize);
            if !(0..128).contains(&ax) || !(0..128).contains(&ay) {
                src_buf[idx] = 0;
            } else {
                src_buf[idx] = memory.sprite_get(ax as u8, ay as u8);
            }
        }
    }

    for py in 0..dh {
        for px in 0..dw {
            let sx_off = (if flip_x { dw - 1 - px } else { px }) * sw / dw;
            let sy_off = (if flip_y { dh - 1 - py } else { py }) * sh / dh;
            if sx_off < 0 || sx_off >= sw || sy_off < 0 || sy_off >= sh {
                continue;
            }
            let buf_idx = (sy_off as usize) * src_w + (sx_off as usize);
            let col = src_buf[buf_idx];
            if is_transparent(memory, col) {
                continue;
            }
            put_pixel_raw(memory, dx + px - cam_x, dy + py - cam_y, col);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn map(
    memory: &mut Memory,
    cel_x: i32,
    cel_y: i32,
    sx: i32,
    sy: i32,
    cel_w: i32,
    cel_h: i32,
    layer: i32,
) {
    let map_w = if memory.ram[0x5F57] == 0 {
        128
    } else {
        memory.ram[0x5F57] as i32
    };
    for cy in 0..cel_h {
        for cx in 0..cel_w {
            let mx = cel_x + cx;
            let my = cel_y + cy;
            if mx < 0 || mx >= map_w || !(0..64).contains(&my) {
                continue;
            }
            let tile = map_get_wide(memory, mx, my);
            if tile == 0 && (memory.ram[0x5F36] & 0x8) == 0 {
                continue;
            }
            if layer != 0 {
                let flags = memory.ram[memory::ADDR_FLAGS as usize + tile as usize];
                if flags & (layer as i8 as u8) == 0 {
                    continue;
                }
            }
            let tile_sx = (tile as i32 % 16) * 8;
            let tile_sy = (tile as i32 / 16) * 8;
            draw_sprite(
                memory,
                tile_sx,
                tile_sy,
                sx + cx * 8,
                sy + cy * 8,
                8,
                8,
                false,
                false,
            );
        }
    }
}

pub fn map_get_wide(memory: &Memory, x: i32, y: i32) -> u8 {
    let map_w = if memory.ram[0x5F57] == 0 {
        128
    } else {
        memory.ram[0x5F57] as i32
    };
    if x < 0 || x >= map_w || !(0..64).contains(&y) {
        return 0;
    }
    if x < 128 {
        return memory.map_get(x as u8, y as u8);
    }
    let addr = ((y as u32 * map_w as u32 + x as u32) & 0xFFFF) as u16;
    let base = memory.ram[0x5F56] as u16;
    let map_base = base * 256;
    memory.ram[map_base.wrapping_add(addr) as usize]
}

pub fn map_set_wide(memory: &mut Memory, x: i32, y: i32, val: u8) {
    let map_w = if memory.ram[0x5F57] == 0 {
        128
    } else {
        memory.ram[0x5F57] as i32
    };
    if x < 0 || x >= map_w || !(0..64).contains(&y) {
        return;
    }
    if x < 128 {
        memory.map_set(x as u8, y as u8, val);
        return;
    }
    let addr = ((y as u32 * map_w as u32 + x as u32) & 0xFFFF) as u16;
    let base = memory.ram[0x5F56] as u16;
    let map_base = base * 256;
    memory.ram[map_base.wrapping_add(addr) as usize] = val;
}

// === Print ===

fn parse_hex_color(c: u8) -> u8 {
    if c.is_ascii_digit() {
        c - b'0'
    } else if (b'a'..=b'f').contains(&c) {
        c - b'a' + 10
    } else if (b'A'..=b'F').contains(&c) {
        c - b'A' + 10
    } else {
        c & 0x0F
    }
}

pub fn draw_text(memory: &mut Memory, text: &[u8], start_x: i32, start_y: i32, col: u8) -> i32 {
    let (cam_x, cam_y) = get_camera(memory);
    let mut x = start_x - cam_x;
    let mut y = start_y - cam_y;
    let mut color = col;
    let mut char_w: i32 = 4;
    let mut char_h: i32 = 6;
    let mut home_x = x;
    let mut home_y = y;
    let mut tab_w: i32 = 16;
    let mut i = 0usize;
    while i < text.len() {
        let ch = text[i];
        i += 1;
        match ch {
            0x01 => {
                if i + 1 < text.len() {
                    let count = text[i];
                    let rch = text[i + 1];
                    i += 2;
                    for _ in 0..count {
                        draw_char(memory, rch, x, y, color);
                        x += char_w;
                    }
                }
            }
            0x02 => {
                if i + 2 < text.len() {
                    let ox = text[i] as i8 as i32;
                    let oy = text[i + 1] as i8 as i32;
                    let dch = text[i + 2];
                    i += 3;
                    draw_char(memory, dch, x + ox, y + oy, color);
                }
            }
            0x03 => {
                if i < text.len() {
                    x += text[i] as i8 as i32;
                    i += 1;
                }
            }
            0x04 => {
                if i < text.len() {
                    y += text[i] as i8 as i32;
                    i += 1;
                }
            }
            0x05 => {
                if i + 1 < text.len() {
                    x = start_x - cam_x + text[i] as i32;
                    y = start_y - cam_y + text[i + 1] as i32;
                    i += 2;
                }
            }
            0x06 => {
                if i < text.len() {
                    let cmd = text[i];
                    i += 1;
                    match cmd {
                        b'w' => char_w = if char_w == 8 { 4 } else { 8 },
                        b't' => char_h = if char_h == 12 { 6 } else { 12 },
                        b'p' => {
                            char_w = 8;
                            char_h = 12;
                        }
                        b'c' if i < text.len() => {
                            let clear_col = parse_hex_color(text[i]);
                            i += 1;
                            let byte = clear_col | (clear_col << 4);
                            let s = memory.screen_base() as usize;
                            let e = (s + 0x2000).min(memory::RAM_SIZE);
                            for k in s..e {
                                memory.ram[k] = byte;
                            }
                            x = start_x - cam_x;
                            y = start_y - cam_y;
                        }
                        b'd' if i < text.len() => {
                            i += 1;
                        }
                        b'g' => {
                            x = home_x;
                            y = home_y;
                        }
                        b'h' => {
                            home_x = x;
                            home_y = y;
                        }
                        b'j' if i + 1 < text.len() => {
                            x += text[i] as i8 as i32;
                            y += text[i + 1] as i8 as i32;
                            i += 2;
                        }
                        b'r' if i < text.len() => {
                            i += 1;
                        }
                        b's' if i < text.len() => {
                            tab_w = (text[i] as i32).max(1);
                            i += 1;
                        }
                        b'x' if i < text.len() => {
                            char_w = text[i] as i32;
                            i += 1;
                        }
                        b'y' if i < text.len() => {
                            char_h = text[i] as i32;
                            i += 1;
                        }
                        _ => {}
                    }
                }
            }
            0x00 => break,
            0x08 => x -= char_w,
            0x09 => x = (x / tab_w) * tab_w + tab_w,
            0x0A => {
                x = start_x - cam_x;
                y += char_h;
            }
            0x0C => {
                if i < text.len() {
                    color = parse_hex_color(text[i]);
                    i += 1;
                }
            }
            0x0D => x = start_x - cam_x,
            0x0E => char_w = 8,
            0x0F => char_w = 4,
            _ => {
                draw_char(memory, ch, x, y, color);
                x += char_w;
            }
        }
    }
    x + cam_x
}

fn draw_char(memory: &mut Memory, code: u8, x: i32, y: i32, col: u8) {
    for py in 0..6u8 {
        for px in 0..4u8 {
            if gfx_font::get_pixel(code, px, py) {
                put_pixel_no_cam(memory, x + px as i32, y + py as i32, col);
            }
        }
    }
}

pub fn print_text_default(state: &mut PicoState, text: &[u8]) -> i32 {
    let cx = state.memory.ram[memory::ADDR_CURSOR_X as usize] as i32;
    let cy = state.memory.ram[memory::ADDR_CURSOR_Y as usize] as i32;
    let col = state.memory.ram[memory::ADDR_COLOR as usize] & 0x0F;
    let right = draw_text(&mut state.memory, text, cx, cy, col);
    state.memory.ram[memory::ADDR_CURSOR_Y as usize] = ((cy + 6) & 0xFF) as u8;
    right
}

pub fn print_text_at(memory: &mut Memory, text: &[u8], x: i32, y: i32, col: u8) -> i32 {
    draw_text(memory, text, x, y, col)
}
