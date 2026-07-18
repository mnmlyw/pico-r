pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Size-golfed carts glue `@`/`%`/`$` directly after a keyword with no
-- separating space (`if$24064~=0then...`). Splicing in `peek(`/`peek2(`/
-- `peek4(` verbatim fused the keyword and the function name into a single
-- bad identifier for the real Lua lexer (`ifpeek4`). Confirmed against a
-- real corpus cart (redash-7.p8.png: `if$24064~=0then?"..."`).
poke4(0x4300,0)
if$0x4300~=1then
printh("RESULT dollar_glue_no")
else
printh("RESULT dollar_glue_yes")
end

poke(0x4300,0)
if@0x4300==0then
printh("RESULT at_glue ok")
end

printh("DONE")
