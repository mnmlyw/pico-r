pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
srand(1)
local co=cocreate(function() end)
printh("RESULT_TYPE_CO "..type(co))
printh("RESULT_TOSTR_CO "..tostr(co))
printh("RESULT_TYPE_NUM "..type(1))
printh("RESULT_TYPE_TABLE "..type({}))
printh("DONE")
