pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- `if`/`elseif` accept `do` as an alternative to `then` -- confirmed
-- against official PICO-8 (unlike `while`, which does NOT accept `then`
-- in place of `do`). Real corpus cart uses this without parens around the
-- condition (build_a_jetpack-1.p8.png: `if gd.lvl <= -4 do ... end`).
local x=2
if x==1 then
  printh("RESULT one")
elseif x==2 do
  printh("RESULT elseif_do ok")
else
  printh("RESULT else")
end

if x>0 do
  printh("RESULT if_do_no_parens ok")
end

if(x>0) do
  printh("RESULT if_do_parens ok")
end

printh("DONE")
