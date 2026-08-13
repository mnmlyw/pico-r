pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pairs() walks the ARRAY part ascending and then the NODE part in raw slot
-- order, so the order depends on where each key hashes, on collision
-- displacement, and on the table's growth history. No sort can reproduce it:
-- the same key set inserted in two different orders comes out differently,
-- which is what the first two cases here pin.
function o(t) local s="" for k,v in pairs(t) do s=s..k.."," end return s end
function build(ks,ds)
	local t={}
	for k in all(ks) do t[k]=1 end
	if ds then for k in all(ds) do t[k]=nil end end
	return t
end
printh("RESULT fwd "..o(build({100,200,300,400})))
printh("RESULT rev "..o(build({400,300,200,100})))
printh("RESULT evens "..o(build({2,4,6,8})))
printh("RESULT dense "..o(build({1,2,3,4,5,6,7,8})))
printh("RESULT densrev "..o(build({8,7,6,5,4,3,2,1})))
printh("RESULT negzero "..o(build({0,-1,-2,-3})))
printh("RESULT mixed "..o(build({1,2,3,1000,-5,17,42})))
printh("RESULT sparse "..o(build({7,49,343,2401})))
-- a nil store to an ABSENT key is not a no-op in Lua 5.2: it creates the
-- entry, which can rehash and even grow the array part
printh("RESULT nilstore "..o(build({5,6},{99})))
-- deleting leaves a dead key that still occupies its slot
printh("RESULT dead "..o(build({10,20,30,40,50},{20,40})))
printh("RESULT revive "..o(build({10,20,30,40,50},{20,40})).."|"..o(build({10,20,30,40,50,20})))
-- the array part never shrinks, so a filled-then-emptied table keeps its shape
printh("RESULT noshrink "..o(build({1,2,3,4,5,6,7,8},{1,2,3,4,5,6,7})))
printh("DONE")
