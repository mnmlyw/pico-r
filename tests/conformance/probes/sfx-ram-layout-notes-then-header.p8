pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- A 68-byte SFX entry is 32 little-endian note words at +0..63 followed by
-- the {mode, speed, loop_start, loop_end} header at +64..67 -- NOT the
-- header first. pico-r's .p8 text loader used to write it header-first,
-- rotating every entry by 4 bytes, so speed/loop were read out of note 0 and
-- every note played two notes late.
--
-- Note word packing (16-bit LE): bits 0-5 pitch, 6-8 waveform, 9-11 volume,
-- 12-14 effect, bit 15 = the waveform's high bit (custom instruments 8-15).
-- Volume and effect are 3 bits, so the text field's high nibble is masked
-- away -- a 'c' volume reads back as 4.
function b(a) printh("RESULT b"..a.." "..peek(0x3200+a)) end
for i=0,7 do b(i) end
for i=62,71 do b(i) end
printh("DONE")
__sfx__
010203040d8c11a7b22b6a33c594000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
1122334455667788000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
