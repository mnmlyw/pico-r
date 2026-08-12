"""Reconstruct a .p8 text cart from a .p8.png and its extracted Lua.

Usage: p8png2p8.py <cart.p8.png> <lua-from-dump-raw> <out.p8>

The differential harness needs to inject a shim into a cart and run the
result on both engines, which means it needs an editable text cart. The ROM
sections come out of the PNG's low two bits per channel (the usual PICO-8
steganography, in a,r,g,b order); the Lua comes from `dump-raw`, which
already handles the compressed-code formats.

The gfx/map/gff/Lua sections were verified by round-tripping corpus carts
through pico-r; the __sfx__/__music__ encoders were NOT verified against the
official binary, so a cart that reads SFX RAM as data may not survive this
round trip faithfully.
"""

import os
import sys

from PIL import Image

png, luafile, out = sys.argv[1], sys.argv[2], sys.argv[3]
im = Image.open(png).convert('RGBA')
px = list(im.convert('RGBA').tobytes())
rom = bytearray()
for i in range(0, len(px), 4):
    r, g, b, a = px[i], px[i + 1], px[i + 2], px[i + 3]
    rom.append(((a & 3) << 6) | ((r & 3) << 4) | ((g & 3) << 2) | (b & 3))

lua = open(luafile, 'rb').read()

def hexrows(start, nrows, rowbytes, swap=False):
    out = []
    for i in range(nrows):
        off = start + i * rowbytes
        row = rom[off:off + rowbytes]
        if swap:
            out.append(''.join('%x%x' % (v & 15, v >> 4) for v in row))
        else:
            out.append(''.join('%02x' % v for v in row))
    return out

def trim(rows):
    while rows and set(rows[-1]) == {'0'}:
        rows.pop()
    return rows

parts = ['pico-8 cartridge // http://www.pico-8.com', 'version 42', '__lua__']
parts.append(lua.decode('latin-1'))
gfx = trim(hexrows(0x0000, 128, 64, swap=True))
if gfx:
    parts.append('__gfx__')
    parts.extend(gfx)
gff = trim(hexrows(0x3000, 2, 128))
if gff:
    parts.append('__gff__')
    parts.extend(gff)
mp = trim(hexrows(0x2000, 32, 128))
if mp:
    parts.append('__map__')
    parts.extend(mp)

# sfx: 64 entries of 68 bytes at 0x3200
sfx = []
for s in range(64):
    off = 0x3200 + s * 68
    blk = rom[off:off + 68]
    if not any(blk):
        sfx.append(None)
        continue
    ed, sp, ls, le = blk[64], blk[65], blk[66], blk[67]
    line = '%02x%02x%02x%02x' % (ed, sp, ls, le)
    for n in range(32):
        lo, hi = blk[n * 2], blk[n * 2 + 1]
        v = lo | (hi << 8)
        pitch = v & 0x3F
        wave = (v >> 6) & 7
        vol = (v >> 9) & 7
        eff = (v >> 12) & 7
        if v & 0x8000:
            wave += 8
        line += '%02x%x%x%x' % (pitch, wave, vol, eff)
    sfx.append(line)
while sfx and sfx[-1] is None:
    sfx.pop()
if sfx:
    parts.append('__sfx__')
    parts.extend(x if x else '0' * 84 for x in sfx)

# music: 64 patterns of 4 bytes at 0x3100
mus = []
for p in range(64):
    off = 0x3100 + p * 4
    b = rom[off:off + 4]
    if not any(b):
        mus.append(None)
        continue
    flags = ((b[0] >> 7) & 1) | (((b[1] >> 7) & 1) << 1) | (((b[2] >> 7) & 1) << 2)
    mus.append('%02x %02x%02x%02x%02x' % (flags, b[0] & 0x7f, b[1] & 0x7f, b[2] & 0x7f, b[3] & 0x7f))
while mus and mus[-1] is None:
    mus.pop()
if mus:
    parts.append('__music__')
    parts.extend(x if x else '00 41424344' for x in mus)

open(out, 'w', encoding='latin-1').write('\n'.join(parts) + '\n')
print('wrote', out, os.path.getsize(out), 'bytes')
