pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- PICO-8's 16.16 fixed point has no infinity/NaN: division by zero
-- saturates to the max/min representable fixed value instead. Mod by
-- zero is always 0 (not derived from the saturated division).
printh("RESULT div_pos_zero "..(1/0))
printh("RESULT div_neg_zero "..(-1/0))
printh("RESULT div_zero_zero "..(0/0))
printh("RESULT intdiv_pos_zero "..(5\0))
printh("RESULT intdiv_neg_zero "..(-5\0))
printh("RESULT mod_pos_zero "..(5%0))
printh("RESULT mod_neg_zero "..(-5%0))
printh("RESULT div_eq_max "..tostr((1/0)==0x7fff.ffff))
printh("DONE")
