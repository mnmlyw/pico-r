pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Results of math API functions should land exactly on the 16.16 grid,
-- not carry extra f64 precision beyond 1/65536.
printh("RESULT sqrt2 "..sqrt(2))
printh("RESULT sqrt2_hex "..tostr(sqrt(2),0x1))
printh("RESULT sqrt3 "..sqrt(3))
printh("RESULT sqrt_frac "..sqrt(0.5))
printh("DONE")
