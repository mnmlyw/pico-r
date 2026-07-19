pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
srand(1)
sfx(1,0)
printh("RESULT before "..stat(16))
sfx(1,-2)
printh("RESULT after "..stat(16))
sfx(2,1)
printh("RESULT ch1_before "..stat(17))
sfx(2,-2)
printh("RESULT ch1_after "..stat(17))
sfx(3,2)
sfx(-2,2)
printh("RESULT ch2_after_negativen "..stat(18))
printh("DONE")
