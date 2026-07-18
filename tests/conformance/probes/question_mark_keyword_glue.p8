pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Size-golfed carts glue `?` directly onto a preceding keyword with no
-- separating space (`then?"x"`). Splicing in `print(` verbatim fuses the
-- keyword and the word `print` into a single identifier for the real Lua
-- lexer (`thenprint`), which fails to parse. Confirmed against a real
-- corpus cart (homunculus-0.p8.png: `if aj>30then?"...",x,y,13`).
local a=30
if a>10then?"x"
printh("RESULT then_glue ok")
end

local b=1
while b>0do?"x"
b=0
printh("RESULT do_glue ok")
end

printh("DONE")
