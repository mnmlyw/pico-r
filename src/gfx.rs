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
        // Pattern bits index MSB-first: bit 15 is pixel (0,0), bit 0 is
        // (3,3) -- oracle-locked by px_rect_fillp's 0b1111111111111110
        // case (only (3,3) is pattern-0 there, not (0,0)).
        let bit_idx = 15 - (py * 4 + px);
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

/// Same fillp/palette pipeline as `put_pixel_raw`, but never consults
/// `clip()` -- circfill/rectfill/ovalfill's "invert" fill mode is
/// oracle-confirmed to always cover the full 128x128 screen regardless of
/// the active clip rect (a clipped and unclipped invert-fill of the same
/// shape produce bit-identical screens).
fn put_pixel_ignore_clip(memory: &mut Memory, sx: i32, sy: i32, col: u8) {
    if !(0..128).contains(&sx) || !(0..128).contains(&sy) {
        return;
    }
    let pat = get_fill_pattern(memory);
    if pat != 0 {
        let px = sx as u32 & 3;
        let py = sy as u32 & 3;
        let bit_idx = 15 - (py * 4 + px);
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

#[allow(clippy::too_many_arguments)]
pub fn rectfill(
    memory: &mut Memory,
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    col: u8,
    invert: bool,
) {
    if x0 > x1 {
        core::mem::swap(&mut x0, &mut x1);
    }
    if y0 > y1 {
        core::mem::swap(&mut y0, &mut y1);
    }
    if invert {
        // Fill everything on the 128x128 screen OUTSIDE [x0,x1]x[y0,y1] --
        // same "invert" semantics as circfill's, and confirmed to likewise
        // bypass clip() entirely.
        let (cam_x, cam_y) = get_camera(memory);
        let (ax0, ay0, ax1, ay1) = (x0 - cam_x, y0 - cam_y, x1 - cam_x, y1 - cam_y);
        for sy in 0..128 {
            if sy < ay0 || sy > ay1 {
                for sx in 0..128 {
                    put_pixel_ignore_clip(memory, sx, sy, col);
                }
            } else {
                for sx in 0..ax0 {
                    put_pixel_ignore_clip(memory, sx, sy, col);
                }
                for sx in (ax1 + 1)..128 {
                    put_pixel_ignore_clip(memory, sx, sy, col);
                }
            }
        }
        return;
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
    draw_circ(memory, cx, cy, r, col, false, false);
}

pub fn circfill(memory: &mut Memory, cx: i32, cy: i32, r: i32, col: u8, invert: bool) {
    draw_circ(memory, cx, cy, r, col, true, invert);
}

fn draw_circ(memory: &mut Memory, cx: i32, cy: i32, r: i32, col: u8, fill: bool, invert: bool) {
    if r < 0 {
        return;
    }
    let (cam_x, cam_y) = get_camera(memory);
    let acx = cx - cam_x;
    let acy = cy - cam_y;
    let invert = fill && invert;

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

/// Per-row half-width of the same midpoint circle `draw_circ` rasterizes,
/// keyed by |dy| from the center -- used so invert-fill's "outside the
/// circle" complement lines up pixel-for-pixel with the boundary the
/// normal (non-invert) fill would have drawn, rather than an independent
/// sqrt-based approximation that can disagree with the Bresenham walk at
/// the edge.
fn circ_row_half_widths(r: i32) -> Vec<i32> {
    let mut widths = vec![-1i32; (r + 1) as usize];
    let mut x: i32 = r;
    let mut y: i32 = 0;
    let mut d: i32 = 1 - r;
    while x >= y {
        widths[y as usize] = widths[y as usize].max(x);
        widths[x as usize] = widths[x as usize].max(y);
        y += 1;
        if d < 0 {
            d += 2 * y + 1;
        } else {
            x -= 1;
            d += 2 * (y - x) + 1;
        }
    }
    widths
}

fn hline(memory: &mut Memory, x0: i32, x1: i32, y: i32, col: u8) {
    let mut sx = x0;
    while sx <= x1 {
        put_pixel_raw(memory, sx, y, col);
        sx += 1;
    }
}

fn invert_fill_circ(memory: &mut Memory, cx: i32, cy: i32, r: i32, col: u8) {
    // Fills the full 128x128 screen outside the circle -- ignores clip()
    // entirely (oracle-confirmed: a clipped and unclipped invert-fill of
    // the same circle produce bit-identical screens). Uses the exact same
    // per-row half-width as the Bresenham walk `draw_circ` itself uses for
    // a normal fill, so the boundary row lines up pixel-for-pixel instead
    // of drifting from an independent sqrt-based circle formula.
    let widths = circ_row_half_widths(r);
    for sy in 0..128 {
        let dy = (sy - cy).abs();
        let half = if dy <= r { widths[dy as usize] } else { -1 };
        if half < 0 {
            for sx in 0..128 {
                put_pixel_ignore_clip(memory, sx, sy, col);
            }
        } else {
            let left = cx - half;
            let right = cx + half;
            for sx in 0..left {
                put_pixel_ignore_clip(memory, sx, sy, col);
            }
            for sx in (right + 1)..128 {
                put_pixel_ignore_clip(memory, sx, sy, col);
            }
        }
    }
}

pub fn oval(memory: &mut Memory, x0: i32, y0: i32, x1: i32, y1: i32, col: u8) {
    draw_oval(memory, x0, y0, x1, y1, col, false, false);
}

pub fn ovalfill(memory: &mut Memory, x0: i32, y0: i32, x1: i32, y1: i32, col: u8, invert: bool) {
    draw_oval(memory, x0, y0, x1, y1, col, true, invert);
}

#[allow(clippy::too_many_arguments)]
fn draw_oval(
    memory: &mut Memory,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    col: u8,
    fill: bool,
    invert: bool,
) {
    let invert = fill && invert;
    // Invert mode fills the full 128x128 screen OUTSIDE the oval, ignoring
    // clip() entirely -- same semantics as circfill/rectfill's invert.
    // Collected as one [left,right] span per screen row (rows with no
    // span are entirely outside the oval), then the complement of each
    // span is filled after the normal walk below records them.
    let mut invert_spans: [Option<(i32, i32)>; 128] = [None; 128];
    let record_span = |spans: &mut [Option<(i32, i32)>; 128], y: i32, xa: i32, xb: i32| {
        if (0..128).contains(&y) {
            spans[y as usize] = Some(match spans[y as usize] {
                Some((la, lb)) => (la.min(xa), lb.max(xb)),
                None => (xa, xb),
            });
        }
    };
    // Exact port of the official rasterizer (reverse-engineered from the
    // PICO-8 binary's draw_oval/draw_filloval + draw_ellipse_1 /
    // fill_ellipse_1): an integer midpoint walk of the quarter arc from
    // (0,b) to (a,0) where cx=(x0+x1)>>1, a=(x1-x0)>>1, px/py the box
    // parities. The error term tracks
    //   err = b^2*x*(x+1) + a^2*y*(y-1) - a^2*b^2
    // and the step thresholds use TRUNCATED quarter squares plus the
    // RADIUS parity (a&1 / b&1) -- both quirks are oracle-locked by a
    // full 1..=32 x 1..=32 size sweep against the official binary
    // (1024/1024 outline + 1024/1024 fill). Verified byte-exact.
    let (cam_x, cam_y) = get_camera(memory);
    let (x0, x1) = (x0.min(x1) - cam_x, x0.max(x1) - cam_x);
    let (y0, y1) = (y0.min(y1) - cam_y, y0.max(y1) - cam_y);
    let w = x1 - x0;
    let h = y1 - y0;
    if h < 2 || w < 2 {
        // Degenerate boxes (thinner than 3px either way) are drawn as
        // FILLED rects by both oval() and ovalfill() -- matches the
        // official hline fallback path.
        if invert {
            for sy in 0..128 {
                if sy < y0 || sy > y1 {
                    for sx in 0..128 {
                        put_pixel_ignore_clip(memory, sx, sy, col);
                    }
                } else {
                    for sx in 0..x0 {
                        put_pixel_ignore_clip(memory, sx, sy, col);
                    }
                    for sx in (x1 + 1)..128 {
                        put_pixel_ignore_clip(memory, sx, sy, col);
                    }
                }
            }
            return;
        }
        for yy in y0..=y1 {
            for xx in x0..=x1 {
                put_pixel_raw(memory, xx, yy, col);
            }
        }
        return;
    }
    let cx = (x0 + x1) >> 1;
    let cy = (y0 + y1) >> 1;
    let a = (w >> 1) as i64;
    let b = (h >> 1) as i64;
    let px = w & 1;
    let py = h & 1;
    let aa = a * a;
    let bb = b * b;
    let thr_x = -(aa / 4 + (a & 1) + bb);
    let thr_y = -(bb / 4 + (b & 1));
    let thr_diag = -(bb / 4 + (b & 1) + aa);
    let mut err = -aa * b;
    let mut dxs: i64 = 0;
    let mut dys = -2 * aa * b;
    let mut x: i64 = 0;
    let mut y: i64 = b;
    let hline = |memory: &mut Memory, xa: i32, xb: i32, yy: i32| {
        for xx in xa..=xb {
            put_pixel_raw(memory, xx, yy, col);
        }
    };
    loop {
        let xi = x as i32;
        let yi = y as i32;
        // y > -py / x > -px guards stop the mirrored halves from
        // re-plotting the shared center row/column when the box has an
        // odd pixel count on that axis.
        let y_mirror = y > 0 || py == 1;
        let x_mirror = x > 0 || px == 1;
        if !fill {
            put_pixel_raw(memory, cx + px + xi, cy + py + yi, col);
            if y_mirror && x_mirror {
                put_pixel_raw(memory, cx - xi, cy - yi, col);
            }
            if y_mirror {
                put_pixel_raw(memory, cx + px + xi, cy - yi, col);
            }
            if x_mirror {
                put_pixel_raw(memory, cx - xi, cy + py + yi, col);
            }
        }
        if x * bb + err > thr_x && y * aa + err > thr_y {
            // About to move down a row: the fill emits this row's spans
            // now, using the current (maximal) x for the row.
            if fill && invert {
                record_span(&mut invert_spans, cy - yi, cx - xi, cx + px + xi);
                if y_mirror {
                    record_span(&mut invert_spans, cy + py + yi, cx - xi, cx + px + xi);
                }
            } else if fill {
                hline(memory, cx - xi, cx + px + xi, cy - yi);
                if y_mirror {
                    hline(memory, cx - xi, cx + px + xi, cy + py + yi);
                }
            }
            if err - y * aa <= thr_diag {
                x += 1;
                dxs += 2 * bb;
                y -= 1;
                dys += 2 * aa;
                err += dxs + dys;
            } else {
                y -= 1;
                dys += 2 * aa;
                err += dys;
            }
        } else {
            x += 1;
            dxs += 2 * bb;
            err += dxs;
        }
        if y < 0 || x > a {
            break;
        }
    }
    if invert {
        for (sy, span) in invert_spans.iter().enumerate() {
            match span {
                None => {
                    for sx in 0..128 {
                        put_pixel_ignore_clip(memory, sx, sy as i32, col);
                    }
                }
                Some((l, r)) => {
                    for sx in 0..*l {
                        put_pixel_ignore_clip(memory, sx, sy as i32, col);
                    }
                    for sx in (*r + 1)..128 {
                        put_pixel_ignore_clip(memory, sx, sy as i32, col);
                    }
                }
            }
        }
    }
}

pub fn rrect(memory: &mut Memory, x0: i32, y0: i32, x1: i32, y1: i32, r: i32, col: u8) {
    draw_rrect(memory, x0, y0, x1, y1, r, col, false);
}

pub fn rrectfill(memory: &mut Memory, x0: i32, y0: i32, x1: i32, y1: i32, r: i32, col: u8) {
    draw_rrect(memory, x0, y0, x1, y1, r, col, true);
}

// PICO-8 0.2.4+ rrect/rrectfill: rrect(x0,y0,x1,y1,r,[col]),
// rrectfill(x0,y0,x1,y1,r,[col]) -- the 5th arg is a corner-rounding
// amount, NOT a color (col is always the optional 6th arg, defaulting to
// the current pen color like every other draw primitive -- confirmed
// against the official binary: rrect(...,7) with no col draws in the
// default pen color 6, and an explicit 6th arg is honored and persists as
// the new pen color same as get_color()'s usual behavior).
//
// Each corner is carved by the SAME midpoint-circle stepping as circ(),
// using an effective radius of r+1. cuts[dy] is the inward pixel offset
// (from the corner) at which row `dy` (0-indexed from the nearest
// horizontal edge, 0..=r) starts being drawn/filled. Oracle-locked by a
// full r=0..15 sweep (plus spot checks up to r=20) against the official
// binary, reading each row's leftmost drawn column.
fn rrect_corner_cuts(r: i32) -> Vec<i32> {
    if r <= 0 {
        return vec![0];
    }
    let big_r = r + 1;
    let mut reach = vec![0i32; (big_r + 1) as usize];
    let mut x = big_r;
    let mut y: i32 = 0;
    let mut d = 1 - big_r;
    while x >= y {
        reach[y as usize] = reach[y as usize].max(x);
        reach[x as usize] = reach[x as usize].max(y);
        y += 1;
        if d < 0 {
            d += 2 * y + 1;
        } else {
            x -= 1;
            d += 2 * (y - x) + 1;
        }
    }
    (0..=r)
        .map(|dy| big_r - reach[(big_r - dy) as usize])
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn draw_rrect(
    memory: &mut Memory,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    r: i32,
    col: u8,
    fill: bool,
) {
    let (cam_x, cam_y) = get_camera(memory);
    let (x0, x1) = (x0.min(x1) - cam_x, x0.max(x1) - cam_x);
    let (y0, y1) = (y0.min(y1) - cam_y, y0.max(y1) - cam_y);
    let w = x1 - x0;
    let h = y1 - y0;
    // Sanity clamp only (exact official clamp behavior for radii larger
    // than the box is unverified): keeps the cuts table small and keeps
    // top/bottom (or left/right) corner bands from crossing entirely.
    let r = r.max(0).min(w.min(h)).min(256);
    let cuts = rrect_corner_cuts(r);
    let cut_at = |dy: i32| -> i32 {
        if dy <= r {
            cuts[dy as usize]
        } else {
            0
        }
    };
    let mut y = y0;
    while y <= y1 {
        let dy = (y - y0).min(y1 - y);
        let cut = cut_at(dy);
        let (lx, rx) = (x0 + cut, x1 - cut);
        if fill || y == y0 || y == y1 {
            hline(memory, lx, rx, y, col);
        } else {
            put_pixel_raw(memory, lx, y, col);
            put_pixel_raw(memory, rx, y, col);
        }
        y += 1;
    }
}

/// tline(x0,y0,x1,y1, mx,my, mdx,mdy, layers) -- textured line.
///
/// All screen coords and map coords are 16.16 fixed point (camera already
/// subtracted by the caller, in fixed point). Exact port of the official
/// draw_tline (reverse-engineered from the binary):
///  - the pixel walk is a DDA over n = max(|flr(x1)-flr(x0)|,
///    |flr(y1)-flr(y0)|) steps (n+1 pixels inclusive), stepping
///    (d<<16)/n per axis from the FLOORED start pixel's center; lines are
///    normalized to walk top-to-bottom (endpoints swapped and mdx/mdy
///    negated, with mx/my advanced to the far end, when y1<y0).
///  - per pixel: map cell = ((mx>>16) & (peek(0x5F38)-1)) + peek(0x5F3A)
///    (mask is reg-1, so 0 = wrap at 256), same for y with 0x5F39/0x5F3B;
///    cell 0 draws nothing unless the 0x5F36 bit 0x8 "draw tile 0" flag is
///    set; a nonzero `layers` mask requires (fget(cell) & layers) != 0;
///    the texel is (cell%16)*8 + ((mx>>13)&7) into the sprite sheet;
///    palt-transparent colors are skipped; then mx += mdx, my += mdy.
#[allow(clippy::too_many_arguments)]
pub fn tline(
    memory: &mut Memory,
    x0f: i32,
    y0f: i32,
    x1f: i32,
    y1f: i32,
    mut mx: i32,
    mut my: i32,
    mut mdx: i32,
    mut mdy: i32,
    layers: i32,
) {
    let x0i = x0f >> 16;
    let y0i = y0f >> 16;
    let x1i = x1f >> 16;
    let y1i = y1f >> 16;
    let mut dx = (x1i - x0i) as i64;
    let mut dy = (y1i - y0i) as i64;
    let n = dx.abs().max(dy.abs());
    let (sx, sy) = if y1i < y0i {
        // Walk top-to-bottom: swap endpoints, advance the map sample to
        // the far end and negate the steps so each pixel samples the same
        // map position it would have in the original direction.
        mx = mx.wrapping_add((n as i32).wrapping_mul(mdx));
        my = my.wrapping_add((n as i32).wrapping_mul(mdy));
        mdx = mdx.wrapping_neg();
        mdy = mdy.wrapping_neg();
        dx = -dx;
        dy = -dy;
        (x1i, y1i)
    } else {
        (x0i, y0i)
    };
    let (step_x, step_y) = if n != 0 {
        ((dx << 16) / n, (dy << 16) / n)
    } else {
        (0, 0)
    };
    let mut xf = ((sx as i64) << 16) + 0x8000;
    let mut yf = ((sy as i64) << 16) + 0x8000;
    let wmask = (memory.ram[0x5F38] as i32).wrapping_sub(1) & 0xFF;
    let hmask = (memory.ram[0x5F39] as i32).wrapping_sub(1) & 0xFF;
    let xoff = memory.ram[0x5F3A] as i32;
    let yoff = memory.ram[0x5F3B] as i32;
    let draw_zero = memory.ram[0x5F36] & 0x8 != 0;
    for _ in 0..=n {
        let cell_x = ((mx >> 16) & wmask) + xoff;
        let cell_y = ((my >> 16) & hmask) + yoff;
        let cell = map_get_wide(memory, cell_x, cell_y) as i32;
        if cell != 0 || draw_zero {
            let layer_ok = layers == 0 || {
                let flags = memory.ram[memory::ADDR_FLAGS as usize + cell as usize];
                (flags as i32) & layers != 0
            };
            if layer_ok {
                let tx = (cell % 16) * 8 + ((mx >> 13) & 7);
                let ty = (cell / 16) * 8 + ((my >> 13) & 7);
                let col = memory.sprite_get(tx as u8, ty as u8);
                if !is_transparent(memory, col) {
                    // fillp() doesn't dither tline() either -- see
                    // draw_sprite's comment.
                    put_pixel_no_cam(memory, (xf >> 16) as i32, (yf >> 16) as i32, col);
                }
            }
        }
        xf += step_x;
        yf += step_y;
        mx = mx.wrapping_add(mdx);
        my = my.wrapping_add(mdy);
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
            // Confirmed against official PICO-8: fillp() never dithers
            // spr()/sspr()/map()/tline() draws (an active fill pattern
            // during any of them produces a bit-identical screen hash to
            // no fillp at all, oracle-locked across plain, secondary-color,
            // and per-pixel-transparent pattern encodings) -- contrary to
            // the manual's documented "apply" bit. `put_pixel_no_cam` is
            // the same clip/palette pipeline as `put_pixel_raw` minus the
            // fill-pattern step.
            put_pixel_no_cam(memory, dx + px - cam_x, dy + py - cam_y, col);
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
    // A negative destination width/height doesn't mean "draw nothing" --
    // it anchors the rectangle at dx/dy and extends it in the negative
    // direction, flipped, e.g. `dw = -8` at `dx = 50` draws the 8 columns
    // `[42, 50)` mirrored. Oracle-locked against px_spr_edge's
    // `sspr(0,0,8,8,50,0,-8,8)`.
    let (dx, dw, flip_x) = if dw < 0 {
        (dx + dw, -dw, !flip_x)
    } else {
        (dx, dw, flip_x)
    };
    let (dy, dh, flip_y) = if dh < 0 {
        (dy + dh, -dh, !flip_y)
    } else {
        (dy, dh, flip_y)
    };

    if dw <= 0 || dh <= 0 || sw <= 0 || sh <= 0 {
        return;
    }
    // No-op once the source rect exceeds the sprite sheet's own 128px
    // dimension -- oracle-confirmed the real threshold is 128, not 256
    // (sw=128 draws normally; sw=129 already no-ops, well below where
    // this used to guard).
    if sw > 128 || sh > 128 {
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

    // Sampling rule (matches the official pico8_stretch_blit, confirmed by
    // disassembly and oracle-locked by px_spr_map's 16x8 -> 8x4 case):
    // 16.16 fixed-point stepping with step = (src<<16)/dst (truncating
    // division) and the accumulator starting at step/2 -- i.e. sampling at
    // the destination pixel CENTER, src = (step/2 + i*step) >> 16. A plain
    // floor(i*src/dst) picks texel 2i instead of 2i+1 on 2:1 downscales.
    let step_x = ((sw as i64) << 16) / dw as i64;
    let step_y = ((sh as i64) << 16) / dh as i64;
    for py in 0..dh {
        for px in 0..dw {
            let ix = (if flip_x { dw - 1 - px } else { px }) as i64;
            let iy = (if flip_y { dh - 1 - py } else { py }) as i64;
            let sx_off = (((step_x >> 1) + ix * step_x) >> 16) as i32;
            let sy_off = (((step_y >> 1) + iy * step_y) >> 16) as i32;
            if sx_off < 0 || sx_off >= sw || sy_off < 0 || sy_off >= sh {
                continue;
            }
            let buf_idx = (sy_off as usize) * src_w + (sx_off as usize);
            let col = src_buf[buf_idx];
            if is_transparent(memory, col) {
                continue;
            }
            // fillp() doesn't dither sspr() -- see draw_sprite's comment.
            put_pixel_no_cam(memory, dx + px - cam_x, dy + py - cam_y, col);
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
    // Confirmed against official PICO-8: a raw 0 here means width 256, not
    // 128 (Memory::new/init_draw_state set the default byte to 128 so a
    // fresh cart still gets the normal 128-wide map).
    let map_w = if memory.ram[memory::ADDR_MAP_WIDTH as usize] == 0 {
        256
    } else {
        memory.ram[memory::ADDR_MAP_WIDTH as usize] as i32
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

/// Resolve the active map region: (base_addr, width, height). With the
/// default base page 0x20 the classic accessors (0x2000 + shared 0x1000)
/// apply and this returns None. A custom base (big maps: `poke(0x5f56,
/// 0x80,0)` = 32KB of upper RAM as a 256-wide, 128-tall map) is row-major
/// at base*256 for ALL columns -- including x<128 -- and its height is
/// however many full rows fit between the base and the end of RAM; both
/// confirmed via oracle (terra_1cart-42.p8.png generates its world there).
fn custom_map_region(memory: &Memory) -> Option<(usize, i32, i32)> {
    let base = memory.ram[0x5F56] as usize;
    if base == 0x20 {
        return None;
    }
    let map_w = if memory.ram[memory::ADDR_MAP_WIDTH as usize] == 0 {
        256
    } else {
        memory.ram[memory::ADDR_MAP_WIDTH as usize] as i32
    };
    let start = base * 256;
    let height = ((memory::RAM_SIZE - start) as i32) / map_w;
    Some((start, map_w, height))
}

pub fn map_get_wide(memory: &Memory, x: i32, y: i32) -> u8 {
    if let Some((start, map_w, height)) = custom_map_region(memory) {
        if x < 0 || x >= map_w || y < 0 || y >= height {
            return 0;
        }
        return memory.ram[start + (y * map_w + x) as usize];
    }
    // Classic map. Confirmed against official PICO-8: a raw 0 width means
    // 256, not 128 (Memory::new/init_draw_state set the default byte to
    // 128 so a fresh cart still gets the normal 128-wide map).
    let map_w = if memory.ram[memory::ADDR_MAP_WIDTH as usize] == 0 {
        256
    } else {
        memory.ram[memory::ADDR_MAP_WIDTH as usize] as i32
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
    if let Some((start, map_w, height)) = custom_map_region(memory) {
        if x < 0 || x >= map_w || y < 0 || y >= height {
            return;
        }
        memory.ram[start + (y * map_w + x) as usize] = val;
        return;
    }
    let map_w = if memory.ram[memory::ADDR_MAP_WIDTH as usize] == 0 {
        256
    } else {
        memory.ram[memory::ADDR_MAP_WIDTH as usize] as i32
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

pub fn draw_text(
    memory: &mut Memory,
    text: &[u8],
    start_x: i32,
    start_y: i32,
    col: u8,
) -> (i32, i32, i32) {
    let (cam_x, cam_y) = get_camera(memory);
    let mut x = start_x - cam_x;
    let mut y = start_y - cam_y;
    let mut color = col;
    let mut char_w: i32 = 4;
    let mut char_h: i32 = 6;
    let mut home_x = x;
    let mut home_y = y;
    let mut tab_w: i32 = 16;
    let mut max_x = x;
    let mut i = 0usize;
    // P8SCII rendering-attribute state (oracle-locked against
    // px_p8scii_modes): pixel-doubling (wide/tall), invert (draw the
    // glyph's OFF pixels in `color`, ON pixels transparent), opaque
    // background colour, and pinball mode. All reset per print() call --
    // confirmed on the golden: a `\^w` in one print() doesn't leak into
    // the next print()'s glyphs.
    let mut wide = false;
    let mut tall = false;
    let mut invert = false;
    let mut bg_color: Option<u8> = None;
    let mut pinball = false;
    // `\v` (0x0B) "decorate previous character": anchor is NOT the current
    // draw cursor but a one-token-delayed snapshot of it (see comment at
    // the 0x0B arm) -- oracle-locked against a chain of `\v` decorations
    // in px_p8scii_modes ("a\vfb\vfc").
    let mut anchor_x = x;
    let mut anchor_y = y;
    // High-water mark of char_h across the whole call -- the caller's
    // FINAL end-of-print cursor-y advance (api_print) uses whichever
    // tall/pinball mode was EVER active during this string, even if
    // explicitly turned back off before the string ends. Embedded `\n`
    // (0x0A) advances use the live char_h at that exact moment instead
    // (unchanged) -- oracle-locked: `print("\^ta\^-ta\nb")` advances the
    // embedded newline by 6 (tall already off) but the final advance by
    // 12 (tall was active earlier in the same call), 18 total.
    let mut max_char_h = char_h;
    while i < text.len() {
        let ch = text[i];
        i += 1;
        max_char_h = max_char_h.max(char_h);
        match ch {
            0x01 => {
                if i + 1 < text.len() {
                    // `\*` param char decodes through the same hex-digit
                    // scheme as `\f` (0-9/a-f -> 0-15), NOT its raw ASCII
                    // byte value -- oracle-locked by px_print_font's
                    // `\*5xy` (repeats "x" 5 times, then draws "y", ~24px
                    // total), not a count of 0x35 flooding the whole line.
                    let count = parse_hex_color(text[i]);
                    let rch = text[i + 1];
                    i += 2;
                    for _ in 0..count {
                        anchor_x = x;
                        anchor_y = y;
                        draw_styled_char(
                            memory, rch, x, y, color, wide, tall, invert, bg_color, pinball,
                        );
                        x += char_width(rch, char_w);
                    }
                }
            }
            // `\#N` (0x02): opaque background colour N behind subsequently
            // printed glyphs. Oracle-locked against px_p8scii_modes'
            // `\#3ij`: the "off" pixels of each glyph paint N instead of
            // staying transparent, and the filled rectangle is 1px WIDER
            // than the glyph box, extending to the left (x-1 .. x+w-1) --
            // confirmed from the golden showing a background-coloured
            // column immediately before the first glyph but nothing after
            // the last one.
            0x02 => {
                if i < text.len() {
                    bg_color = Some(parse_hex_color(text[i]));
                    i += 1;
                }
            }
            // `\-N` (0x03): draw the NEXT character only, offset
            // horizontally by a signed-nibble amount (0-7 -> 0..7, 8-15 ->
            // -8..-1), then resume normal advance from the UNshifted
            // cursor -- oracle-locked against px_p8scii_modes' `m\-fn`
            // ('n' rendered 1px left of its normal slot, 'f' -> -1).
            // Unlike `\v`, this reads the CURRENT cursor directly (no
            // anchor lag).
            0x03 => {
                if i < text.len() {
                    let m = parse_hex_color(text[i]) as i32;
                    i += 1;
                    let dx = if m >= 8 { m - 16 } else { m };
                    if i < text.len() {
                        let dch = text[i];
                        i += 1;
                        anchor_x = x;
                        anchor_y = y;
                        draw_styled_char(
                            memory,
                            dch,
                            x + dx,
                            y,
                            color,
                            wide,
                            tall,
                            invert,
                            bg_color,
                            pinball,
                        );
                        x += char_width(dch, char_w);
                    }
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
                        // `\^w` / `\^t`: pixel-double glyphs horizontally /
                        // vertically (nearest-neighbour 2x), toggled on
                        // each use. Oracle-locked against px_p8scii_modes:
                        // `\^wab` widens each source pixel to 2px (advance
                        // doubles to 8, hence char_w flips too); `\^tcd`
                        // stretches each glyph row to 2 output rows.
                        b'w' => {
                            wide = !wide;
                            char_w = if wide { 8 } else { 4 };
                        }
                        b't' => {
                            tall = !tall;
                            char_h = if tall { 12 } else { 6 };
                        }
                        // `\^i`: invert -- draw the glyph's OFF pixels in
                        // `color` and leave ON pixels transparent, i.e. a
                        // photographic negative confined to the glyph's
                        // own box (no bleed into the advance-cell margin).
                        b'i' => invert = !invert,
                        // `\^p`: "pinball" mode -- the glyph is doubled
                        // 2x2 like `\^w\^t` but only the top-right
                        // subpixel of each 2x2 block is drawn, giving the
                        // sparse dot-matrix look (each ON font pixel
                        // (px,py) emits exactly one screen pixel at
                        // (2*px+1, 2*py); advance doubles like wide).
                        // Decoded from the px_p8scii_modes golden: its
                        // dot offsets are all odd-x/even-y, and mapping
                        // them back through /2 reproduces the `s` and `t`
                        // font bitmaps exactly.
                        b'p' => {
                            pinball = !pinball;
                            char_w = if pinball { 8 } else { 4 };
                            char_h = if pinball { 12 } else { 6 };
                        }
                        // `\^-X`: explicitly clear (not toggle) mode X.
                        // Oracle-locked against px_p8scii_modes'
                        // `\^wop\^-wqr`: wide is ON for "op", OFF for "qr".
                        b'-' if i < text.len() => {
                            let sub = text[i];
                            i += 1;
                            match sub {
                                b'w' => {
                                    wide = false;
                                    char_w = 4;
                                }
                                b't' => {
                                    tall = false;
                                    char_h = 6;
                                }
                                b'i' => invert = false,
                                b'p' => {
                                    pinball = false;
                                    char_w = 4;
                                    char_h = 6;
                                }
                                _ => {}
                            }
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
            // `\a` (0x07): audio-cue annotation -- everything from here to
            // the end of the STRING is the audio encoding, not visible
            // text (oracle-confirmed: print("\ac1x") advances the cursor
            // by zero; print("ab\acc") only "ab" contributes to the
            // return value/cursor advance, "cc" is swallowed). Not just
            // "this call" like 0x00 -- there's no further dispatch after
            // it either way since it consumes the rest of `text`.
            0x07 => break,
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
            // `\vN` (0x0B): "decorate previous character" -- draw the NEXT
            // character offset from an ANCHOR point on a 4x4 grid (x in
            // the low 2 bits, spanning -2..+1; y in the high bits, -8
            // upward in steps of 1) without moving the print cursor.
            // Oracle-locked against px_p8scii_modes' `a\vfb\vfc`: the
            // anchor is NOT the live cursor but a one-token-delayed
            // snapshot of it -- the first decoration after a normal
            // character anchors at THAT character's own position, while a
            // second (or later) decoration in a row anchors at the
            // cursor's current (frozen, since decorations never move it)
            // position. Modelled by updating `anchor_{x,y}` to the
            // pre-token cursor snapshot after every glyph-producing token
            // (normal or decorated) -- see the default arm and the 0x03
            // arm above for the same pattern.
            0x0B => {
                if i < text.len() {
                    let m = parse_hex_color(text[i]) as i32;
                    i += 1;
                    if i < text.len() {
                        let dch = text[i];
                        i += 1;
                        let dx = (m % 4) - 2;
                        let dy = -8 + (m / 4);
                        let cur_x = x;
                        let cur_y = y;
                        draw_styled_char(
                            memory,
                            dch,
                            anchor_x + dx,
                            anchor_y + dy,
                            color,
                            wide,
                            tall,
                            invert,
                            bg_color,
                            pinball,
                        );
                        anchor_x = cur_x;
                        anchor_y = cur_y;
                    }
                }
            }
            _ => {
                anchor_x = x;
                anchor_y = y;
                draw_styled_char(
                    memory, ch, x, y, color, wide, tall, invert, bg_color, pinball,
                );
                x += char_width(ch, char_w);
            }
        }
        // print()'s return value is the right-most x reached at any point
        // while printing, not wherever a trailing newline left the cursor
        // -- confirmed against official PICO-8 ("hello\nb" returns the x
        // after "hello", not after "b").
        max_x = max_x.max(x);
    }
    max_char_h = max_char_h.max(char_h);
    (max_x + cam_x, y + cam_y, max_char_h)
}

// Wide P8SCII glyphs (0x80-0xff) always advance the cursor by a fixed 8px
// (7px of art + 1px spacing), ignoring the narrow-glyph char_w setting --
// oracle-locked by px_print_font's "\x83\x86\x88" line, whose 3 glyphs sit
// 8px apart regardless of the (default 4px) char_w in effect.
fn char_width(code: u8, char_w: i32) -> i32 {
    if code >= 0x80 {
        8
    } else {
        char_w
    }
}

// General glyph blit honouring the P8SCII rendering-attribute state
// (wide/tall pixel-doubling, invert, opaque background, pinball). With
// every flag at its default (false/false/false/None/false) this produces
// pixel-identical output to the old unconditional draw_char, so existing
// callers (repeat, plain chars) are unaffected when no mode is active.
#[allow(clippy::too_many_arguments)]
fn draw_styled_char(
    memory: &mut Memory,
    code: u8,
    x: i32,
    y: i32,
    col: u8,
    wide: bool,
    tall: bool,
    invert: bool,
    bg: Option<u8>,
    pinball: bool,
) {
    if pinball {
        // Sparse dot-matrix rendering: one screen pixel per ON font
        // pixel, at the top-right subpixel of the 2x2-doubled block
        // (see the `\^p` handler comment for the golden decode).
        let (bw, bh): (u8, u8) = if code >= 0x80 { (8, 8) } else { (4, 6) };
        for py in 0..bh {
            for px in 0..bw {
                let on = if code >= 0x80 {
                    gfx_font::get_wide_pixel(code, px, py)
                } else {
                    gfx_font::get_pixel(code, px, py)
                };
                if on {
                    put_pixel_no_cam(memory, x + px as i32 * 2 + 1, y + py as i32 * 2, col);
                }
            }
        }
        return;
    }

    let (bw, bh): (u8, u8) = if code >= 0x80 { (8, 8) } else { (4, 6) };
    let mult_x = if wide { 2 } else { 1 };
    let mult_y = if tall { 2 } else { 1 };

    // `on_color`/`off_color`: what to paint a glyph pixel that's ON (a lit
    // font pixel) vs OFF. Plain glyph: on -> `col`, off -> nothing.
    // Opaque background (`\#N`): on -> `col`, off -> the background
    // colour (oracle-locked against `\#3ij`). Invert (`\^i`): on ->
    // nothing, off -> `col` -- a photographic negative (oracle-locked
    // against `\^igh`). Both `\#N` and `\^i` additionally paint a column
    // 1px to the LEFT of the glyph box with the off-colour (confirmed on
    // both goldens: a background/inverted pixel appears immediately
    // before the first glyph, but nothing trails after the last one).
    let (on_color, off_color): (Option<u8>, Option<u8>) = if invert {
        (None, Some(col))
    } else {
        (Some(col), bg)
    };

    if let Some(oc) = off_color {
        let box_h = bh as i32 * mult_y;
        for yy in 0..box_h {
            put_pixel_no_cam(memory, x - 1, y + yy, oc);
        }
    }

    for py in 0..bh {
        for px in 0..bw {
            let on = if code >= 0x80 {
                gfx_font::get_wide_pixel(code, px, py)
            } else {
                gfx_font::get_pixel(code, px, py)
            };
            let color_to_use = if on { on_color } else { off_color };
            if let Some(c) = color_to_use {
                let ox = x + px as i32 * mult_x;
                let oy = y + py as i32 * mult_y;
                for dy in 0..mult_y {
                    for dx in 0..mult_x {
                        put_pixel_no_cam(memory, ox + dx, oy + dy, c);
                    }
                }
            }
        }
    }
}
