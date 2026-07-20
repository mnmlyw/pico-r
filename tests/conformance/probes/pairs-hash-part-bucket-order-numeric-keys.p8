pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: pairs() order for NUMERIC keys living in the hash part. Unlike
-- string keys (whose order is seed-random per run), numeric keys hash
-- seedlessly, so their bucket order IS deterministic -- verified
-- byte-identical across 4 spaced oracle runs. Reproducing it requires
-- porting lua 5.2 ltable.c's node layout (luai_hashnum on the 16.16
-- raw bits, mainposition/collision chains, power-of-2 rehash growth
-- history) -- the orders below are bucket order, NOT numeric or
-- insertion order: {100,1,50} iterates 1,100,50 and {-1,0,0.5,1}
-- iterates 0,-1,0.5,1 (the 1 lands in the hash part here because the
-- node array never grew past 4).
local function order(t)
 local s="" for k,v in pairs(t) do s=s..tostr(k)..","end return s
end

local s3={}
s3[100]=1 s3[1]=1 s3[50]=1
printh("RESULT sparse_c "..order(s3))

local n={}
n[-1]=1 n[0]=1 n[0.5]=1 n[1]=1
printh("RESULT numkeys "..order(n))
printh("DONE")
