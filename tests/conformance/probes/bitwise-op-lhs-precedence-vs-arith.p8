pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
work_ram=100
printh("RESULT shift_add "..(work_ram+2<<5))
printh("RESULT and_add "..(5+3&12))
printh("RESULT or_add "..(1+2|8))
printh("RESULT sub_shr "..(10-2>>1))
printh("RESULT mul_bxor "..(2*3^^1))
printh("DONE")
