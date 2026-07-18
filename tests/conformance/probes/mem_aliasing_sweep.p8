pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Broader sweep across the shared sprite/map region to rule out an
-- edge case the minimal probe missed.
local mismatches=0
for i=0,20 do
  local mx=flr(rnd(128))
  local my=32+flr(rnd(32))
  local val=1+flr(rnd(15))
  mset(mx,my,val)
  local sy=64+(my-32)*2+flr(mx/64)
  local sx=mx%64*2
  -- verify via sget at the two nibbles the byte should cover
  local lo=sget(sx,sy)
  local hi=sget(sx+1,sy)
  local combined=lo+hi*16
  if combined~=val then
    mismatches+=1
    printh("RESULT mismatch_"..i.." mx="..mx.." my="..my.." val="..val.." combined="..combined)
  end
end
printh("RESULT total_mismatches "..mismatches)
printh("DONE")
