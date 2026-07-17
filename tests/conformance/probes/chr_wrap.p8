pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- chr() wraps out-of-range ordinals via % 256 rather than dropping them.
local s=chr(256,300,-1,257,512)
printh("RESULT b1 "..ord(s,1))
printh("RESULT b2 "..ord(s,2))
printh("RESULT b3 "..ord(s,3))
printh("RESULT b4 "..ord(s,4))
printh("RESULT b5 "..ord(s,5))
printh("DONE")
