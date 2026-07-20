pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: the DETERMINISTIC subset of pairs() iteration order. The
-- array part (dense 1..n) always iterates first, ascending, before any
-- hash-part key; hash-part STRING key order is genuinely random per run
-- on the real console (verified: 4 runs of the same cart give 4
-- different permutations -- z8lua seeds its string hash from
-- time(NULL)+ASLR addresses), so string keys are only asserted as a
-- sorted SET here. Every line below was verified byte-identical across
-- 4 spaced oracle runs before capture.
local function sortjoin(t)
 local ks={}
 for k,v in pairs(t) do add(ks,tostr(k)) end
 for i=1,#ks do for j=i+1,#ks do if ks[j]<ks[i] then ks[i],ks[j]=ks[j],ks[i] end end end
 local s="" for i=1,#ks do s=s..ks[i].."," end return s
end
local function order(t)
 local s="" for k,v in pairs(t) do s=s..tostr(k)..","end return s
end

local t={}
t[3]="c" t.name="n" t[1]="a" t[2]="b" t.other="o"
local o=order(t)
printh("RESULT dense_prefix "..sub(o,1,6))
printh("RESULT dense_set "..sortjoin(t))

local u={} u[2]=1 u[1]=1 u[3]=1 u[4]=1
printh("RESULT pure_dense "..order(u))

local c={10,20,30,x=1}
printh("RESULT ctor_prefix "..sub(order(c),1,6))

-- sparse positive ints (no strings): numerically ascending regardless
-- of insertion order (1,2,3 land in the array part; 7,9 in the hash
-- part whose numeric-hash bucket order happens to be ascending here --
-- see pairs-hash-part-bucket-order-numeric-keys for cases where bucket
-- order is NOT ascending)
local s={}
s[1]=1 s[2]=1 s[3]=1 s[7]=1 s[9]=1
printh("RESULT sparse_a "..order(s))

local s2={}
s2[9]=1 s2[7]=1 s2[3]=1 s2[2]=1 s2[1]=1
printh("RESULT sparse_b "..order(s2))
printh("DONE")
