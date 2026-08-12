pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- split()'s first argument is documented as a string. Does real PICO-8
-- raise a runtime error (aborting the cart) when it's nil/omitted, or
-- silently treat it as an empty string like pico-r currently does?
printh("RESULT before")
local t=split(nil)
printh("RESULT after "..#t)
local t2=split()
printh("RESULT omitted "..#t2)
printh("DONE")
