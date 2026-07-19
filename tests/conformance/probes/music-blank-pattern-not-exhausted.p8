pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
music(0)
printh("RESULT stat24_after_music0 "..stat(24))
printh("RESULT stat57_after_music0 "..tostr(stat(57)))
music(-1)
printh("RESULT stat24_after_stop "..stat(24))
printh("RESULT stat57_after_stop "..tostr(stat(57)))
printh("DONE")
