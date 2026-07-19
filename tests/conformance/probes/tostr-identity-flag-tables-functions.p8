pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- tostr identity flag (0x1) on tables/functions
local t={}
local r=tostr(t,1)
printh("RESULT t_prefix "..sub(r,1,6))
printh("RESULT t_space "..sub(r,7,7))
printh("RESULT t_0x "..sub(r,8,9))
printh("RESULT t_len_ge "..tostr(#r>=10))

local function f() end
local r2=tostr(f,1)
printh("RESULT f_prefix "..sub(r2,1,9))
printh("RESULT f_space "..sub(r2,10,10))
printh("RESULT f_0x "..sub(r2,11,12))
printh("RESULT f_len_ge "..tostr(#r2>=13))

-- edge: default flags (0) unaffected by this gap
printh("RESULT t_default "..tostr(t))
printh("RESULT f_default "..tostr(f))

-- edge: flag 0x1 on a number still works (sanity, unrelated code path)
printh("RESULT n_flag1 "..tostr(1.5,1))
printh("DONE")
