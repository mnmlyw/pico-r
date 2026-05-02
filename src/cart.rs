use crate::memory::{self, Memory};

pub struct Cart {
    pub lua_code: String,
}

#[derive(Debug)]
pub enum CartError {
    InvalidPng,
    UnsupportedPng,
    InvalidCompression,
    InvalidPxa,
    Zlib,
}

pub fn load_bytes(data: &[u8], memory: &mut Memory) -> Result<Cart, CartError> {
    if data.len() >= 8 && &data[0..4] == b"\x89PNG" {
        parse_p8_png(data, memory)
    } else {
        parse_p8_text(data, memory)
    }
}

fn hex_val(ch: u8) -> u8 {
    match ch {
        b'0'..=b'9' => ch - b'0',
        b'a'..=b'f' => ch - b'a' + 10,
        b'A'..=b'F' => ch - b'A' + 10,
        _ => 0,
    }
}

fn parse_p8_text(content: &[u8], memory: &mut Memory) -> Result<Cart, CartError> {
    let mut lua_lines: Vec<u8> = Vec::new();

    #[derive(Copy, Clone, PartialEq)]
    enum Section {
        None,
        Lua,
        Gfx,
        Gff,
        Map,
        Sfx,
        Music,
        Label,
    }

    let mut section = Section::None;
    let mut section_line: usize = 0;

    for raw_line in split_lines(content) {
        let line = strip_cr(raw_line);

        if starts_with(line, b"__lua__") {
            section = Section::Lua;
            section_line = 0;
            continue;
        } else if starts_with(line, b"__gfx__") {
            section = Section::Gfx;
            section_line = 0;
            continue;
        } else if starts_with(line, b"__gff__") {
            section = Section::Gff;
            section_line = 0;
            continue;
        } else if starts_with(line, b"__map__") {
            section = Section::Map;
            section_line = 0;
            continue;
        } else if starts_with(line, b"__sfx__") {
            section = Section::Sfx;
            section_line = 0;
            continue;
        } else if starts_with(line, b"__music__") {
            section = Section::Music;
            section_line = 0;
            continue;
        } else if starts_with(line, b"__label__") {
            section = Section::Label;
            section_line = 0;
            continue;
        }

        match section {
            Section::Lua => {
                lua_lines.extend_from_slice(line);
                lua_lines.push(b'\n');
            }
            Section::Gfx => {
                if section_line < 128 {
                    parse_gfx_line(memory, line, section_line);
                }
                section_line += 1;
            }
            Section::Gff => {
                if section_line < 2 {
                    parse_gff_line(memory, line, section_line);
                }
                section_line += 1;
            }
            Section::Map => {
                if section_line < 32 {
                    parse_map_line(memory, line, section_line);
                }
                section_line += 1;
            }
            Section::Sfx => {
                if section_line < 64 {
                    parse_sfx_line(memory, line, section_line);
                }
                section_line += 1;
            }
            Section::Music => {
                if section_line < 64 {
                    parse_music_line(memory, line, section_line);
                }
                section_line += 1;
            }
            Section::None | Section::Label => {}
        }
    }

    Ok(Cart {
        lua_code: String::from_utf8_lossy(&lua_lines).into_owned(),
    })
}

fn split_lines(data: &[u8]) -> impl Iterator<Item = &[u8]> {
    data.split(|&b| b == b'\n')
}

fn strip_cr(line: &[u8]) -> &[u8] {
    if line.last() == Some(&b'\r') {
        &line[..line.len() - 1]
    } else {
        line
    }
}

fn starts_with(line: &[u8], prefix: &[u8]) -> bool {
    line.len() >= prefix.len() && &line[..prefix.len()] == prefix
}

fn parse_gfx_line(memory: &mut Memory, line: &[u8], row: usize) {
    let y = row as u8;
    let mut x: u8 = 0;
    while (x as usize) < 128 && (x as usize) < line.len() {
        let color = hex_val(line[x as usize]);
        memory.sprite_set(x, y, color);
        x += 1;
    }
}

fn parse_gff_line(memory: &mut Memory, line: &[u8], row: usize) {
    let base = memory::ADDR_FLAGS as usize + row * 128;
    let mut i = 0;
    while i + 1 < line.len() && i / 2 < 128 {
        let hi = hex_val(line[i]);
        let lo = hex_val(line[i + 1]);
        memory.ram[base + i / 2] = (hi << 4) | lo;
        i += 2;
    }
}

fn parse_map_line(memory: &mut Memory, line: &[u8], row: usize) {
    let base = memory::ADDR_MAP as usize + row * 128;
    let mut i = 0;
    while i + 1 < line.len() && i / 2 < 128 {
        let hi = hex_val(line[i]);
        let lo = hex_val(line[i + 1]);
        memory.ram[base + i / 2] = (hi << 4) | lo;
        i += 2;
    }
}

fn parse_sfx_line(memory: &mut Memory, line: &[u8], row: usize) {
    if line.len() < 8 {
        return;
    }
    let base = memory::ADDR_SFX as usize + row * 68;
    let speed = (hex_val(line[2]) << 4) | hex_val(line[3]);
    let loop_start = (hex_val(line[4]) << 4) | hex_val(line[5]);
    let loop_end = (hex_val(line[6]) << 4) | hex_val(line[7]);

    memory.ram[base] = (hex_val(line[0]) << 4) | hex_val(line[1]);
    memory.ram[base + 1] = speed;
    memory.ram[base + 2] = loop_start;
    memory.ram[base + 3] = loop_end;

    for note_i in 0..32 {
        let off = 8 + note_i * 5;
        if off + 5 > line.len() {
            break;
        }
        let d0 = hex_val(line[off]);
        let d1 = hex_val(line[off + 1]);
        let d2 = hex_val(line[off + 2]);
        let d3 = hex_val(line[off + 3]);
        let d4 = hex_val(line[off + 4]);
        let pitch = (d0 << 4) | d1;
        let waveform = d2 & 0x7;
        let custom = (d2 >> 3) & 0x1;
        let volume = d3 & 0x7;
        let effect = d4 & 0x7;

        let byte0 = (pitch & 0x3F) | ((waveform & 0x3) << 6);
        let byte1 = ((waveform >> 2) & 0x1) | (volume << 1) | (effect << 4) | (custom << 7);

        let note_addr = base + 4 + note_i * 2;
        memory.ram[note_addr] = byte0;
        memory.ram[note_addr + 1] = byte1;
    }
}

fn parse_music_line(memory: &mut Memory, line: &[u8], row: usize) {
    if line.len() < 11 {
        return;
    }
    let base = memory::ADDR_MUSIC as usize + row * 4;
    let flags = (hex_val(line[0]) << 4) | hex_val(line[1]);

    for ch in 0..4 {
        let off = 3 + ch * 2;
        if off + 1 >= line.len() {
            break;
        }
        let sfx_id = (hex_val(line[off]) << 4) | hex_val(line[off + 1]);
        let disabled = if sfx_id >= 0x40 { 0x40 } else { 0 };
        memory.ram[base + ch] = (sfx_id & 0x3F) | disabled;
    }

    if flags & 0x1 != 0 {
        memory.ram[base] |= 0x80;
    }
    if flags & 0x2 != 0 {
        memory.ram[base + 1] |= 0x80;
    }
    if flags & 0x4 != 0 {
        memory.ram[base + 2] |= 0x80;
    }
}

// === .p8.png ===

fn parse_p8_png(data: &[u8], memory: &mut Memory) -> Result<Cart, CartError> {
    let pixels = decode_png(data)?;

    let total_pixels: usize = 160 * 205;
    let mut cart_data = vec![0u8; total_pixels];
    for i in 0..total_pixels {
        let r = pixels[i * 4];
        let g = pixels[i * 4 + 1];
        let b = pixels[i * 4 + 2];
        let a = pixels[i * 4 + 3];
        cart_data[i] = ((a & 3) << 6) | ((r & 3) << 4) | ((g & 3) << 2) | (b & 3);
    }

    let data_end = 0x4300usize;
    memory.ram[..data_end].copy_from_slice(&cart_data[..data_end]);

    // Rearrange SFX header layout: native PNG format has notes first, header last.
    for sfx_i in 0..64u16 {
        let base = memory::ADDR_SFX as usize + (sfx_i as usize) * 68;
        let hdr = [
            memory.ram[base + 64],
            memory.ram[base + 65],
            memory.ram[base + 66],
            memory.ram[base + 67],
        ];
        // Shift 64 bytes of note data forward by 4 (copy backwards to handle overlap)
        for i in (0..64).rev() {
            memory.ram[base + 4 + i] = memory.ram[base + i];
        }
        memory.ram[base] = hdr[0];
        memory.ram[base + 1] = hdr[1];
        memory.ram[base + 2] = hdr[2];
        memory.ram[base + 3] = hdr[3];
    }

    let lua_region = &cart_data[0x4300..0x8000];
    let lua_code = decompress_lua(lua_region)?;

    Ok(Cart {
        lua_code: String::from_utf8_lossy(&lua_code).into_owned(),
    })
}

fn decode_png(data: &[u8]) -> Result<Vec<u8>, CartError> {
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(CartError::InvalidPng);
    }

    let mut pos = 8usize;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut idat: Vec<u8> = Vec::new();

    while pos + 12 <= data.len() {
        let chunk_len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let chunk_type = &data[pos + 4..pos + 8];
        let chunk_start = pos + 8;
        let chunk_end = chunk_start + chunk_len;
        if chunk_end + 4 > data.len() {
            return Err(CartError::InvalidPng);
        }
        if chunk_type == b"IHDR" {
            if chunk_len < 13 {
                return Err(CartError::InvalidPng);
            }
            width = u32::from_be_bytes([
                data[chunk_start],
                data[chunk_start + 1],
                data[chunk_start + 2],
                data[chunk_start + 3],
            ]);
            height = u32::from_be_bytes([
                data[chunk_start + 4],
                data[chunk_start + 5],
                data[chunk_start + 6],
                data[chunk_start + 7],
            ]);
            let bit_depth = data[chunk_start + 8];
            let color_type = data[chunk_start + 9];
            if width != 160 || height != 205 || bit_depth != 8 || color_type != 6 {
                return Err(CartError::UnsupportedPng);
            }
        } else if chunk_type == b"IDAT" {
            idat.extend_from_slice(&data[chunk_start..chunk_end]);
        } else if chunk_type == b"IEND" {
            break;
        }
        pos = chunk_end + 4;
    }

    if width == 0 || idat.is_empty() {
        return Err(CartError::InvalidPng);
    }

    // Decompress zlib
    let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&idat).map_err(|_| CartError::Zlib)?;

    let bpp: usize = 4;
    let row_bytes = width as usize * bpp;
    let h = height as usize;
    if raw.len() != h * (1 + row_bytes) {
        return Err(CartError::InvalidPng);
    }

    let mut pixels = vec![0u8; h * row_bytes];
    for y in 0..h {
        let filter = raw[y * (1 + row_bytes)];
        let scanline_start = y * (1 + row_bytes) + 1;
        let scanline = &raw[scanline_start..scanline_start + row_bytes];

        for x in 0..row_bytes {
            let raw_byte = scanline[x];
            let a_val: u8 = if x >= bpp {
                pixels[y * row_bytes + x - bpp]
            } else {
                0
            };
            let b_val: u8 = if y > 0 {
                pixels[(y - 1) * row_bytes + x]
            } else {
                0
            };
            let c_val: u8 = if y > 0 && x >= bpp {
                pixels[(y - 1) * row_bytes + x - bpp]
            } else {
                0
            };
            pixels[y * row_bytes + x] = match filter {
                0 => raw_byte,
                1 => raw_byte.wrapping_add(a_val),
                2 => raw_byte.wrapping_add(b_val),
                3 => raw_byte.wrapping_add(((a_val as u16 + b_val as u16) / 2) as u8),
                4 => raw_byte.wrapping_add(paeth(a_val, b_val, c_val)),
                _ => return Err(CartError::InvalidPng),
            };
        }
    }
    Ok(pixels)
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let pa = (p - a as i16).abs();
    let pb = (p - b as i16).abs();
    let pc = (p - c as i16).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn decompress_lua(region: &[u8]) -> Result<Vec<u8>, CartError> {
    if region.len() >= 4 && region[0] == 0x00 && &region[1..4] == b"pxa" {
        return decompress_pxa(region);
    }
    if region.len() >= 4 && &region[0..4] == b":c:\0" {
        return decompress_old(region);
    }
    // Plaintext, null-terminated
    let end = region.iter().position(|&b| b == 0).unwrap_or(region.len());
    Ok(region[..end].to_vec())
}

const OLD_TABLE: &[u8] = b"\n 0123456789abcdefghijklmnopqrstuvwxyz!#%(){}[]<>+=/*:;.,~_";

fn decompress_old(region: &[u8]) -> Result<Vec<u8>, CartError> {
    if region.len() < 8 {
        return Err(CartError::InvalidCompression);
    }
    let decomp_len = ((region[4] as usize) << 8) | region[5] as usize;
    let mut output = vec![0u8; decomp_len];
    let mut out_pos = 0usize;
    let mut i = 8usize;
    while out_pos < decomp_len && i < region.len() {
        let byte = region[i];
        i += 1;
        if byte == 0 {
            if i >= region.len() {
                break;
            }
            output[out_pos] = region[i];
            out_pos += 1;
            i += 1;
        } else if byte <= 0x3b {
            output[out_pos] = OLD_TABLE[(byte - 1) as usize];
            out_pos += 1;
        } else {
            if i >= region.len() {
                break;
            }
            let next = region[i];
            i += 1;
            let ref_offset = (byte - 0x3c) as usize * 16 + (next & 0x0f) as usize;
            let length = (next >> 4) as usize + 2;
            if ref_offset > out_pos {
                return Err(CartError::InvalidCompression);
            }
            for _ in 0..length {
                if out_pos >= decomp_len {
                    break;
                }
                output[out_pos] = output[out_pos - ref_offset];
                out_pos += 1;
            }
        }
    }
    output.truncate(out_pos);
    Ok(output)
}

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bit_pos: 0,
        }
    }
    fn read_bit(&mut self) -> Result<u8, CartError> {
        if self.pos >= self.data.len() {
            return Err(CartError::InvalidPxa);
        }
        let bit = (self.data[self.pos] >> self.bit_pos) & 1;
        if self.bit_pos == 7 {
            self.bit_pos = 0;
            self.pos += 1;
        } else {
            self.bit_pos += 1;
        }
        Ok(bit)
    }
    fn read_bits(&mut self, n: u8) -> Result<u32, CartError> {
        let mut result: u32 = 0;
        for i in 0..n {
            let b = self.read_bit()? as u32;
            result |= b << i;
        }
        Ok(result)
    }
    fn read_byte(&mut self) -> Result<u8, CartError> {
        if self.bit_pos == 0 {
            if self.pos >= self.data.len() {
                return Err(CartError::InvalidPxa);
            }
            let b = self.data[self.pos];
            self.pos += 1;
            return Ok(b);
        }
        Ok(self.read_bits(8)? as u8)
    }
}

fn decompress_pxa(region: &[u8]) -> Result<Vec<u8>, CartError> {
    if region.len() < 8 {
        return Err(CartError::InvalidPxa);
    }
    let decomp_len = u16::from_be_bytes([region[4], region[5]]) as usize;
    let compressed = &region[8..];

    let mut mtf: [u8; 256] = [0; 256];
    for (i, slot) in mtf.iter_mut().enumerate() {
        *slot = i as u8;
    }

    let mut output = vec![0u8; decomp_len];
    let mut out_pos = 0usize;

    let mut reader = BitReader::new(compressed);

    while out_pos < decomp_len {
        let block_type = match reader.read_bit() {
            Ok(b) => b,
            Err(_) => break,
        };

        if block_type == 1 {
            let mut extra: u8 = 0;
            while let Ok(b) = reader.read_bit() {
                if b == 0 {
                    break;
                }
                extra += 1;
            }
            let base = ((1usize << extra) - 1) << 4;
            let n_bits = 4 + extra;
            let raw = reader.read_bits(n_bits)? as usize;
            let idx = raw + base;
            if idx >= 256 {
                return Err(CartError::InvalidPxa);
            }
            let ch = mtf[idx];
            // Move-to-front
            for j in (1..=idx).rev() {
                mtf[j] = mtf[j - 1];
            }
            mtf[0] = ch;
            if out_pos < decomp_len {
                output[out_pos] = ch;
                out_pos += 1;
            }
        } else {
            let first = reader.read_bit()?;
            let offlen: u8 = if first == 1 {
                let second = reader.read_bit()?;
                if second == 1 {
                    5
                } else {
                    10
                }
            } else {
                15
            };
            let mut ref_offset: usize = 0;
            for i in 0..offlen {
                let b = reader.read_bit()? as usize;
                ref_offset |= b << i;
            }
            ref_offset += 1;

            if offlen == 10 && ref_offset == 1 {
                while out_pos < decomp_len {
                    let byte = reader.read_byte()?;
                    if byte == 0 {
                        break;
                    }
                    output[out_pos] = byte;
                    out_pos += 1;
                }
                continue;
            }

            let mut length: usize = 3;
            loop {
                let chunk = reader.read_bits(3)? as usize;
                length += chunk;
                if chunk != 7 {
                    break;
                }
            }

            if ref_offset > out_pos {
                return Err(CartError::InvalidPxa);
            }
            for _ in 0..length {
                if out_pos >= decomp_len {
                    break;
                }
                output[out_pos] = output[out_pos - ref_offset];
                out_pos += 1;
            }
        }
    }
    output.truncate(out_pos);
    Ok(output)
}
