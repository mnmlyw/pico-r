pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- header: fixed char width=8, height=8; fill every glyph bitmap with 0xff
-- so any glyph the custom font renders comes out as a solid 8x8 block
poke(0x5600,8)
poke(0x5602,8)
for a=0x5608,0x5dff do poke(a,0xff) end

cls()
cursor(0,0)
local r=print("\014A")
local s=0
for y=0,7 do for x=0,7 do s=s+(pget(x,y)~=0 and 1 or 0) end end
printh("RESULT custom_glyph_litpixels "..s)
printh("RESULT retval_custom "..r)

cls()
cursor(0,0)
local r2=print("\015A")
local s2=0
for y=0,7 do for x=0,7 do s2=s2+(pget(x,y)~=0 and 1 or 0) end end
printh("RESULT default_glyph_litpixels "..s2)
printh("RESULT retval_default "..r2)

printh("DONE")
