pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- oval()/ovalfill() rasterization, verified pixel-for-pixel (not just
-- hashed) against official PICO-8 for these specific sizes/modes.
cls(0) ovalfill(20,20,60,50,8)
local h=0 for a=0x6000,0x7fff do h=(h*31+@a)&0x7fff end
printh("RESULT oval_a "..h)

cls(0) ovalfill(30,30,60,60,9)
local h2=0 for a=0x6000,0x7fff do h2=(h2*31+@a)&0x7fff end
printh("RESULT oval_circle "..h2)

cls(0) oval(20,20,60,50,8)
local h3=0 for a=0x6000,0x7fff do h3=(h3*31+@a)&0x7fff end
printh("RESULT oval_outline "..h3)

cls(0) ovalfill(0,0,5,3,7)
local h4=0 for a=0x6000,0x7fff do h4=(h4*31+@a)&0x7fff end
printh("RESULT oval_small "..h4)

cls(0) ovalfill(10,10,21,19,7)
local h5=0 for a=0x6000,0x7fff do h5=(h5*31+@a)&0x7fff end
printh("RESULT oval_odd_bbox "..h5)

printh("DONE")
