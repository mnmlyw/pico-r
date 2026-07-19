pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
cls()
cursor(0,0)
print("top")
cursor(0,100)
for i=1,8 do
	print("row"..i)
end
printh("RESULT cursor_y "..peek(0x5f27))
local topsum=0
for y=0,5 do
	for x=0,7 do
		topsum=topsum+pget(x,y)
	end
end
printh("RESULT top_sum "..topsum)
local botsum=0
for y=122,127 do
	for x=0,7 do
		botsum=botsum+pget(x,y)
	end
end
printh("RESULT bot_sum "..botsum)
printh("DONE")
