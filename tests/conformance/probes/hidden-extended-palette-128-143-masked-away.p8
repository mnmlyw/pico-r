pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: pal(c0,c1,1) (screen palette) and pal(c0,c1) (draw palette) both
-- store c1 into the 0x5f00/0x5f10 register tables. Community docs describe
-- a documented hidden 32-color extended palette selected by setting the
-- 0x80 bit on c1 (c1 = 128+n picks extended color n, gated by
-- poke(0x5f2e,1)). pico-r's api_pal masks every c1 write to `& 0xF` before
-- storing, which would destroy that bit (and any value above 15)
-- regardless of what the real console does with it. This probe reads the
-- register straight back via peek to see exactly what byte the real
-- console keeps.

pal()

-- plain in-range value, p=1 (screen pal) and p=0 (draw pal) -- sanity
-- baseline, should read back unchanged either way
pal(9,0x0a,1)
printh("RESULT screenpal9_plain "..peek(0x5f19))
pal()
pal(9,0x0a)
printh("RESULT drawpal9_plain "..peek(0x5f09))
pal()

-- bit 0x10 set (this bit means "transparent" in the draw-pal byte layout,
-- but 0x5f10+ has no such meaning for screen pal) -- does it survive?
pal(9,0x1a,1)
printh("RESULT screenpal9_bit10 "..peek(0x5f19))
pal()

-- bit 0x80 set (the claimed hidden-palette selector bit), low nibble 10,
-- no 0x5f2e gate poked yet
pal(9,0x8a,1)
printh("RESULT screenpal9_bit80_nogate "..peek(0x5f19))
pal()

-- same, but with the claimed gate poked on first
poke(0x5f2e,1)
pal(9,0x8a,1)
printh("RESULT screenpal9_bit80_gate "..peek(0x5f19))
pal()
poke(0x5f2e,0)

-- 128+20 = extended color 20 selector, no gate
pal(9,0x94,1)
printh("RESULT screenpal9_ext20_nogate "..peek(0x5f19))
pal()

-- 128+20, with gate poked
poke(0x5f2e,1)
pal(9,0x94,1)
printh("RESULT screenpal9_ext20_gate "..peek(0x5f19))
pal()
poke(0x5f2e,0)

-- full byte 0xff, p=1
pal(9,0xff,1)
printh("RESULT screenpal9_ff "..peek(0x5f19))
pal()

-- same battery against the draw palette (p=0)
pal(9,0x8a)
printh("RESULT drawpal9_bit80 "..peek(0x5f09))
pal()

pal(9,0x94)
printh("RESULT drawpal9_ext20 "..peek(0x5f09))
pal()

pal(9,0xff)
printh("RESULT drawpal9_ff "..peek(0x5f09))
pal()

-- 0x5f2e itself: does it read back what was poked, plain memory semantics?
poke(0x5f2e,1)
printh("RESULT peek_5f2e_after_poke1 "..peek(0x5f2e))
poke(0x5f2e,0)
printh("RESULT peek_5f2e_after_poke0 "..peek(0x5f2e))

printh("DONE")
