pico-8 cartridge // http://www.pico-8.com
version 41
__lua__
function _init()
 x=42
end
function _update()
 x=x+1
end
function _draw()
 cls()
 print(x,2,2,7)
end
__gfx__
0123456789abcdef
