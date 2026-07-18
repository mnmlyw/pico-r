pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
mset(3,3,7)
poke(0x5f56,0x80,0)
mset(3,3,42)
mset(200,100,9)
printh("RESULT big_lowxy "..mget(3,3))
printh("RESULT big_hixy "..mget(200,100))
printh("RESULT big_ram "..peek(0x8000+100*256+200))
printh("RESULT big_lowram "..peek(0x8000+3*256+3))
printh("RESULT big_oob_y "..mget(0,128))
poke(0x5f56,0x20,128)
printh("RESULT back_std "..mget(3,3))
printh("RESULT std_oob "..mget(3,100))
printh("DONE")
