pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Font table regression lock: representative sample of the full 224-glyph
-- oracle dump that replaced gfx_font.rs (uppercase renders as a distinct
-- "small caps" set -- row 0 blank -- NOT the same bitmap as lowercase).
cls(0) print("Hello, World! 123 @#$%")
local h=0 for a=0x6000,0x7fff do h=(h*31+@a)&0x7fff end
printh("RESULT full_text_hash "..h)

cls(0) print("abcdefghijklmnopqrstuvwxyz")
local h2=0 for a=0x6000,0x7fff do h2=(h2*31+@a)&0x7fff end
printh("RESULT lowercase_hash "..h2)

cls(0) print("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
local h3=0 for a=0x6000,0x7fff do h3=(h3*31+@a)&0x7fff end
printh("RESULT uppercase_hash "..h3)

printh("DONE")
