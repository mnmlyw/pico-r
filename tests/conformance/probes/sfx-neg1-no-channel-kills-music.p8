pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
srand(1)
music(0)
sfx(1,0)
flip()
printh("RESULT before_pattern "..stat(24))
printh("RESULT before_chan0_sfx "..stat(16))
sfx(-1)
flip()
printh("RESULT after_pattern "..stat(24))
printh("RESULT after_chan0_sfx "..stat(16))
music(0)
sfx(1,0)
flip()
sfx(-1,0)
flip()
printh("RESULT chanstop_pattern "..stat(24))
printh("RESULT chanstop_chan0_sfx "..stat(16))
printh("DONE")
__music__
00 01404040

