pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- ? print-shorthand glued to preceding code (no separating space/newline)
-- must still be recognized and produce valid, executable Lua. Verified via
-- printh (not screen pixels -- print()'s pixel-level rendering has a
-- separate, unrelated fidelity gap from this preprocessor transform).
-- `?`'s args always run to the end of the physical line (comments
-- excepted); official PICO-8 also fails to compile if real code is meant
-- to follow `?args` on the same line, so that pattern is intentionally
-- not exercised here.
if true then
  ?"a"
end
local t={7,8,9}
local sum=0
for i=1,#t do
  ?t[i]
  sum+=t[i]
end
do
  ?"d" -- trailing comment must not be swallowed
end
sum+=0 ?"glued after a full statement on one line"
printh("RESULT sum "..sum)
printh("DONE")
