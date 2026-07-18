pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
printh("RESULT s108 "..stat(108))
serial(2056,0,8)
printh("RESULT s108b "..stat(108))
for i=1,100 do serial(2056,0,8) end
printh("RESULT s108c "..stat(108))
printh("DONE")
