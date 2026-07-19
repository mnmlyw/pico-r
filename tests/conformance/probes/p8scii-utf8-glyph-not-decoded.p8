pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
local s="❎"
printh("RESULT LEN1 "..#s)
printh("RESULT B1_1 "..ord(s,1))
local t="a❎b"
printh("RESULT LEN2 "..#t)
printh("RESULT B1_2 "..ord(t,1))
printh("RESULT B2_2 "..ord(t,2))
printh("RESULT B3_2 "..ord(t,3))
local u="⧗"
printh("RESULT LEN3 "..#u)
printh("RESULT B1_3 "..ord(u,1))
printh("DONE")
