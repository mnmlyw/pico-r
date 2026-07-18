pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- split()'s token->number conversion accepts hex (0x, with fraction) and
-- binary (0b) literals, and ignores surrounding whitespace when parsing
-- (" 6" -> 6) while a token that fails to convert keeps its original
-- spacing as a string -- all oracle-confirmed. Real corpus cart:
-- mer_ork-0.p8.png (passes addresses as u"0x6000,0xe000,0x1fff" and
-- spaced args as u"5, 6, 16, 0, 7, 96").
local t=split"0x10,2,-3,0b11,.5,4e2"
for i=1,#t do printh("RESULT a"..i.." "..tostr(t[i]).." "..type(t[i])) end
local w=split"5, 6, 16 ,x , 0x10"
for i=1,#w do printh("RESULT b"..i.." ["..tostr(w[i]).."] "..type(w[i])) end
printh("DONE")
