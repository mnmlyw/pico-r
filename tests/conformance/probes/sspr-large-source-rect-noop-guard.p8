pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
srand(1)

-- fill entire sprite sheet with a solid non-transparent color (1)
for i=0,0x1fff do
 poke(i,0x11)
end

local function hash_screen()
 local h=0
 for i=0,8191 do
  h=(h+peek(0x6000+i)*(i+1))%99999999
 end
 return h
end

-- source width at the sheet boundary (128): should draw normally
cls(0)
sspr(0,0,128,8,0,0,60,8)
printh("RESULT sspr_sw128 "..hash_screen())

-- one past the sheet boundary (129): official no-ops here already,
-- well below the 256 threshold pico-r guards on
cls(0)
sspr(0,0,129,8,0,0,60,8)
printh("RESULT sspr_sw129 "..hash_screen())

-- exactly pico-r's guard threshold (256): still a no-op in official
cls(0)
sspr(0,0,256,20,0,0,60,20)
printh("RESULT sspr_sw256 "..hash_screen())

-- one past pico-r's guard threshold (257)
cls(0)
sspr(0,0,257,20,0,0,60,20)
printh("RESULT sspr_sw257 "..hash_screen())

-- far past the threshold
cls(0)
sspr(0,0,300,20,0,0,60,20)
printh("RESULT sspr_sw300 "..hash_screen())

-- same probes on the sh (source height) axis
cls(0)
sspr(0,0,8,128,0,0,8,60)
printh("RESULT sspr_sh128 "..hash_screen())

cls(0)
sspr(0,0,8,129,0,0,8,60)
printh("RESULT sspr_sh129 "..hash_screen())

cls(0)
sspr(0,0,20,257,0,0,20,60)
printh("RESULT sspr_sh257 "..hash_screen())

printh("DONE")
