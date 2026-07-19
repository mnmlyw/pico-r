pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
function hash_screen()
 local h=0
 for i=0,8191 do h=(h+peek(0x6000+i)*(i+1))%99999999 end
 return h
end

cls()
clip(-5,-5,20,20)
rectfill(0,0,127,127,8)
clip()
printh("RESULT screen_hash_both_neg "..hash_screen())

cls()
clip(-5,5,20,20)
rectfill(0,0,127,127,8)
clip()
printh("RESULT screen_hash_x_neg "..hash_screen())

cls()
clip(5,-5,20,20)
rectfill(0,0,127,127,8)
clip()
printh("RESULT screen_hash_y_neg "..hash_screen())

printh("DONE")
