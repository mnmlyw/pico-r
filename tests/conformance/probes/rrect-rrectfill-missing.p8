pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
cls()
rrect(10,10,50,30,7)
rrectfill(60,10,100,30,8,4)
printh("RESULT rrect_outline_top "..tostring(pget(30,10)))
printh("RESULT rrect_outline_corner "..tostring(pget(11,11)))
printh("RESULT rrectfill_top "..tostring(pget(80,10)))
printh("RESULT rrectfill_fill "..tostring(pget(80,20)))
printh("RESULT rrectfill_corner "..tostring(pget(61,11)))
printh("DONE")
