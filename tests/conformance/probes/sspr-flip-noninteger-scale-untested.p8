pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
for a=0,0x3ff do poke(a,(a*11+a\16)%256) end

cls()
sspr(0,0,8,8,20,20,13,5,true,false)
local h=0 for i=0,8191 do h=(h+peek(0x6000+i)*(i+1))%99999999 end
printh("RESULT sspr_flipx_scale_8to13x8to5_hash "..h)

cls()
sspr(0,0,8,8,20,20,5,13,false,true)
h=0 for i=0,8191 do h=(h+peek(0x6000+i)*(i+1))%99999999 end
printh("RESULT sspr_flipy_scale_8to5x8to13_hash "..h)

cls()
sspr(0,0,11,7,15,15,17,9,true,true)
h=0 for i=0,8191 do h=(h+peek(0x6000+i)*(i+1))%99999999 end
printh("RESULT sspr_flipxy_scale_11to17x7to9_hash "..h)

printh("DONE")
