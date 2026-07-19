pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
srand(1)
cls()
local r1=print("\ac1x")
printh("RESULT r1 "..tostr(r1))
printh("RESULT cx1 "..peek(0x5f26))
printh("RESULT cy1 "..peek(0x5f27))

cls()
local r2=print("ab\acc")
printh("RESULT r2 "..tostr(r2))
printh("RESULT cx2 "..peek(0x5f26))
printh("RESULT cy2 "..peek(0x5f27))

cls()
local r3=print("\a")
printh("RESULT r3 "..tostr(r3))
printh("RESULT cx3 "..peek(0x5f26))
printh("RESULT cy3 "..peek(0x5f27))
printh("DONE")
