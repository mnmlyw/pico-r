pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
cls()
-- establish a valid previous endpoint via two-arg line()
line(5,5,50,5,8)
-- baseline: implicit-continuation line should connect from (50,5)
line(60,60,7)
local function hash()
	local h=0
	for i=0,8191 do h=(h+peek(0x6000+i)*(i+1))%99999999 end
	return h
end
printh("RESULT hash_after_continue "..hash())

cls()
line(5,5,50,5,8)
-- documented poke-to-invalidate trick: should suppress the connecting segment
poke(0x5f35,1)
line(70,70,7)
printh("RESULT hash_after_poke_invalidate "..hash())
printh("RESULT peek_5f35 "..peek(0x5f35))

cls()
line(5,5,50,5,8)
-- explicit 4-arg line() should also reset continuation regardless of flag
line(10,10,20,20,9)
line(90,90,7)
printh("RESULT hash_after_explicit_reset "..hash())

printh("DONE")
