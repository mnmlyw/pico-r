pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- A condition can legitimately continue past a parenthesized prefix via
-- comparison or indexing before its real `then` -- an over-narrow fix for
-- the short-if separator check (checking only the token immediately after
-- the first balanced paren group) regressed both of these. Confirmed
-- against real corpus carts (sheeple-0.p8.png: `if (t()-x) > 0.1 then`;
-- kaizoleste-1.p8.png: `if ({...})[k] then`).
local x=5
if (x-2) > 2 then
  printh("RESULT paren_prefix_comparison ok")
end

local t={[3]=true}
if ({[3]=true})[x-2] then
  printh("RESULT paren_prefix_index ok")
end

printh("DONE")
