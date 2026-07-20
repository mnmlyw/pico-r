pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: does elapsed time (t()/time()) advance BEFORE _update runs each
-- frame, or only AFTER _update/_draw finish? Also touches the very first
-- tick, which happens even before _init's own body runs (a separate,
-- already-fixed "boot tick" -- this probe is about the per-frame ordering
-- from then on).
local n=0
local done=false
function _init()
 printh("RESULT init_t "..tostr(t()))
end
function _update()
 if done then return end
 n+=1
 printh("RESULT update_t_"..n.." "..tostr(t()))
 if n==3 then
  done=true
  printh("DONE")
 end
end
function _draw()
end
