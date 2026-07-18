pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- sfx(id, ch, offset): an out-of-range offset clamps to the valid note
-- range (31), it doesn't fall back to note 0.
sfx(5,0,99)
printh("RESULT offset_oob_note "..stat(20))

-- music(pattern, fade, mask): an EXPLICIT mask of 0 means "reserve no
-- channels", distinct from an omitted mask (which reserves all 4) --
-- pico-r previously conflated the two, so sfx() couldn't use any
-- channel even when the cart explicitly asked for zero reservation.
music(-1)
music(0,0,0)
sfx(5)
printh("RESULT mask0_ch0 "..stat(16))
printh("RESULT mask0_ch1 "..stat(17))
printh("RESULT mask0_ch2 "..stat(18))
printh("RESULT mask0_ch3 "..stat(19))

printh("DONE")
