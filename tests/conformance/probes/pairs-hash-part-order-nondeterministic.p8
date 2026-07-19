pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
local t={}
t[3]="c"
t.name="n"
t[1]="a"
t[2]="b"
t.other="o"
local order=""
for k,v in pairs(t) do order=order..tostring(k)..","end
printh("RESULT order "..order)

local u={}
u.zeta=1
u.alpha=2
u.gamma=3
u.beta=4
local order2=""
for k,v in pairs(u) do order2=order2..tostring(k)..","end
printh("RESULT order2 "..order2)

local v={}
v[1]="a"
v[2]="b"
v[3]="c"
local order3=""
for k,val in pairs(v) do order3=order3..tostring(k)..","end
printh("RESULT order3 "..order3)

printh("DONE")
