pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
local x=sub(nil,1,2)
printh("RESULT type1 "..type(x))
printh("RESULT val1 ["..tostr(x).."]")

local y=sub(nil,1)
printh("RESULT type2 "..type(y))
printh("RESULT val2 ["..tostr(y).."]")

local z=sub("hello",1,2)
printh("RESULT type3 "..type(z))
printh("RESULT val3 ["..z.."]")
printh("DONE")
