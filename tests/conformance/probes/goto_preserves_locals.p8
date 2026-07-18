pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- A backward goto must close only the locals declared AFTER the target
-- label -- locals declared before it stay in scope. The old implementation
-- truncated ALL of the block's locals on any goto, so a loop-via-goto that
-- read a local declared above its label saw nil (hotwax-5.p8.png:
-- `local lin=tb[y]` before `::light_ov::`, jumped back to from inside a
-- pairs() loop).
do
 local x=5
 local n=0
 ::top::
 n+=1
 if n<3 then goto top end
 printh("RESULT preserved "..tostr(x).." "..n)
end

-- Forward goto skips declarations without disturbing existing locals.
do
 local a=1
 goto fwd
 a=99
 ::fwd::
 printh("RESULT forward "..a)
end
printh("DONE")
