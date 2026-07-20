pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: does elapsed time (t()/time()) advance BEFORE _update runs each
-- frame, or only AFTER _update/_draw finish? Checked as the FIRST
-- statement of the frame's own _update call so this is observable within
-- a single frame (this project's conformance harness always runs probes
-- for exactly 1 frame). Also touches the very first tick, which happens
-- even before _init's own body runs (a separate, already-fixed "boot
-- tick" -- this probe is about the per-frame ordering from then on).
-- `done` guards against printing again on frames beyond the first --
-- official PICO-8 keeps calling _update every real-time frame regardless
-- of what Lua does, so this is what keeps the captured output to exactly
-- one RESULT/DONE pair.
local done=false
function _init()
 printh("RESULT init_t "..tostr(t()))
end
function _update()
 if done then return end
 done=true
 printh("RESULT update_t "..tostr(t()))
 printh("DONE")
end
function _draw()
end
