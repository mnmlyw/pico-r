pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- A compound-assign RHS that leaves a paren open at end of line
-- continues on later physical lines (`bx+=d_mbtn(` with the args
-- below). The rewrite must not close its wrapping paren at the first
-- line's end. Real corpus carts: build_a_jetpack-1.p8.png,
-- pony9000_1_3_3-0.p8.png (`pod.x+=(` ... `)*0.1` across lines).
local function three(a,b,c) return a+b+c end
local bx=10
bx+=three(
	1,
	2,
	3)
printh("RESULT call_across_lines "..bx)

local x=5
x+=(
	x*2
)
printh("RESULT paren_across_lines "..x)

printh("DONE")
