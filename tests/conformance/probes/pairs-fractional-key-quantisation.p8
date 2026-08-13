pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- A number key's identity is its 16.16 raw, and the conversion TRUNCATES.
-- 1.00001*65536 is 65536.66 -> 0x0001.0000, while 1.00002*65536 is 65537.31
-- -> 0x0001.0001, so those are two different keys. Rounding instead of
-- truncating collapses them onto one raw and the table appears to hold a
-- single key -- which also silently corrupts pairs() order for every
-- fractional-keyed table, since the layout is computed from those raws.
a,b = tonum("1.00001"), tonum("1.00002")
printh("RESULT raws "..tostr(a,1).." "..tostr(b,1).." equal="..tostr(a==b))
local t={} t[a]=1 t[b]=2
local n=0 for k,v in pairs(t) do n+=1 end
printh("RESULT distinct_keys "..n)

-- ordering over fractional keys, which is what the truncation actually feeds
local u={}
for k in all(split("16.1698 -27.1255 6.6692 -41.2079 11.0405 0.5751 -0.4271"," ")) do u[k]=1 end
local s="" for k,v in pairs(u) do s=s..tostr(k,1).."," end
printh("RESULT frac_order "..s)

-- and the boundary where truncation and rounding disagree in both directions
local w={}
for k in all(split("2.000005 2.000025 -2.000005 -2.000025"," ")) do w[k]=1 end
local s2="" for k,v in pairs(w) do s2=s2..tostr(k,1).."," end
printh("RESULT boundary "..s2)
printh("DONE")
