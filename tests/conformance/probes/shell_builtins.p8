pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- official exposes terminal commands as Lua globals; carts use them as
-- values (redash-7 uses `help` as a no-op default callback)
for _,n in ipairs(split"help,ls,dir,cd,mkdir,folder,save,info,run,reboot,shutdown,exit,keyconfig,splore,install_demos,install_games") do
 printh("RESULT "..n..":"..type(_ENV[n] or false))
end
printh("DONE")
