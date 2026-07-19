pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
function f()
 local n=select("#",...)
 printh("RESULT n "..n)
end
f(1,2,3)
printh("DONE")
