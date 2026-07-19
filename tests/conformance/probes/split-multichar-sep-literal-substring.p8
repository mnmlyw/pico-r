pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
srand(1)
local t1=split("a-b_c","-_")
printh("RESULT LEN1 "..#t1)
printh("RESULT T1_1 "..tostr(t1[1]))
printh("RESULT T1_2 "..tostr(t1[2]))
printh("RESULT T1_3 "..tostr(t1[3]))

local t2=split("a-b_c","_-")
printh("RESULT LEN2 "..#t2)
printh("RESULT T2_1 "..tostr(t2[1]))
printh("RESULT T2_2 "..tostr(t2[2]))
printh("RESULT T2_3 "..tostr(t2[3]))

local t3=split("a,b,,c",",")
printh("RESULT LEN3 "..#t3)
printh("RESULT T3_1 "..tostr(t3[1]))
printh("RESULT T3_2 "..tostr(t3[2]))
printh("RESULT T3_3 "..tostr(t3[3]))
printh("RESULT T3_4 "..tostr(t3[4]))

local t4=split("abc","")
printh("RESULT LEN4 "..#t4)
printh("RESULT T4_1 "..tostr(t4[1]))
printh("RESULT T4_2 "..tostr(t4[2]))
printh("RESULT T4_3 "..tostr(t4[3]))
printh("DONE")
