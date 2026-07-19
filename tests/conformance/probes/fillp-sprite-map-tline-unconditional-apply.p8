pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
function hash()
	local h=0
	for i=0,8191 do h=(h+peek(0x6000+i)*(i+1))%99999999 end
	return h
end

for y=0,7 do for x=0,15 do sset(x,y,8) end end

cls() spr(0,0,0)
printh("RESULT spr_no_fillp "..hash())

cls() fillp(0b0011001111001100) spr(0,0,0) fillp()
printh("RESULT spr_plain_fillp "..hash())

cls() fillp(0b0011001111001100.1) spr(0,0,0) fillp()
printh("RESULT spr_transparent_fillp "..hash())

for x=0,15 do for y=0,15 do mset(x,y,0) end end

cls() map(0,0,0,0,16,16)
printh("RESULT map_no_fillp "..hash())

cls() fillp(0b0011001111001100) map(0,0,0,0,16,16) fillp()
printh("RESULT map_plain_fillp "..hash())

cls() sspr(0,0,8,8,0,0,32,32)
printh("RESULT sspr_no_fillp "..hash())

cls() fillp(0b0011001111001100) sspr(0,0,8,8,0,0,32,32) fillp()
printh("RESULT sspr_plain_fillp "..hash())

printh("DONE")
