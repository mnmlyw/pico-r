pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
local s=chr(false,65)
printh("RESULT LEN "..#s)
printh("RESULT B1 "..ord(s,1))
printh("RESULT B2 "..(ord(s,2) or -1))

local s2=chr("abc")
printh("RESULT LEN2 "..#s2)
printh("RESULT B3 "..(ord(s2,1) or -1))

local s3=chr(65,nil,66)
printh("RESULT LEN3 "..#s3)
printh("RESULT B4 "..(ord(s3,1) or -1))
printh("RESULT B5 "..(ord(s3,2) or -1))
printh("RESULT B6 "..(ord(s3,3) or -1))

printh("DONE")
