pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
local log={}
local proxy=setmetatable({},{__newindex=function(t,k,v) log[#log+1]=tostring(k).."="..tostring(v) rawset(t,k,v) end})
proxy.x=5
printh("RESULT nidx "..(log[1] or "MISSING"))
printh("RESULT nidx_raw "..tostring(rawget(proxy,"x")))

local mt={__eq=function(a,b) return true end,__lt=function(a,b) return a.v<b.v end,__len=function(t) return 99 end,__tostring=function(t) return "V("..tostring(t.v)..")" end}
local a=setmetatable({v=1},mt)
local b=setmetatable({v=2},mt)
printh("RESULT eq "..tostring(a==b))
printh("RESULT lt "..tostring(a<b))
printh("RESULT len "..tostring(#a))
printh("RESULT ts "..tostring(a))

-- nearby edge: tables without metatables should keep raw semantics
local plain1={v=1}
local plain2={v=1}
printh("RESULT plain_eq "..tostring(plain1==plain2))
printh("RESULT plain_eq_self "..tostring(plain1==plain1))
printh("RESULT plain_len "..tostring(#plain1))

-- edge: __eq should not be consulted for tables with different metatables in
-- some lua versions, but 5.2 only requires either operand's __eq; check same-mt case above is enough
printh("DONE")
