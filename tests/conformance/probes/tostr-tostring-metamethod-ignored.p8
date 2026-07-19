pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
local mt={__tostring=function(t) return "V("..tostring(t.v)..")" end}
local a=setmetatable({v=1},mt)
printh("RESULT ts "..tostring(a))
printh("RESULT ts_tostr "..tostr(a))

-- nearby edge: table without __tostring keeps generic placeholder
local plain={v=1}
printh("RESULT ts_plain "..tostring(plain))

-- nearby edge: __tostring returning a non-string-y value coerced via tostring semantics
local mt2={__tostring=function(t) return "N="..t.n end}
local b=setmetatable({n=42},mt2)
printh("RESULT ts_num "..tostring(b))

printh("DONE")
