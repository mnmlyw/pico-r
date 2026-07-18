pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
local co=cocreate(function(a,b)
 printh("RESULT in_co "..a.." "..b)
 local x,y=yield(a+b)
 printh("RESULT resumed_with "..x.." "..y)
 yield(x*y)
 return 99
end)
printh("RESULT status1 "..costatus(co))
local ok,v=coresume(co,3,4)
printh("RESULT first "..tostr(ok).." "..v)
ok,v=coresume(co,5,6)
printh("RESULT second "..tostr(ok).." "..v)
printh("RESULT status2 "..costatus(co))
ok,v=coresume(co)
printh("RESULT third "..tostr(ok).." "..tostr(v))
printh("RESULT status3 "..costatus(co))
ok,v=coresume(co)
printh("RESULT dead "..tostr(ok).." "..tostr(v))
local ok2,err=coresume(cocreate(function() error_undefined_fn() end))
printh("RESULT errco "..tostr(ok2))
printh("DONE")
