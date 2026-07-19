pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: does pget() respect the active camera() offset like pset()?
cls()

-- case 1: negative camera offset (screen = world - cam, so screen > world)
camera(-20,-30)
pset(5,5,9)
local a=pget(5,5)        -- worldspace read while cam active -> should see 9
local b=pget(25,35)      -- rawscreen read while cam active -> should be background (0)
camera()
local c=pget(25,35)      -- rawscreen read after cam reset -> should see 9 (pixel is physically here)
local d=pget(5,5)        -- worldspace coord after cam reset, now equals rawscreen -> background (0)

printh("RESULT pget_worldspace_while_cam_neg "..a)
printh("RESULT pget_rawscreen_while_cam_neg "..b)
printh("RESULT pget_rawscreen_after_reset_neg "..c)
printh("RESULT pget_worldspace_after_reset_neg "..d)

-- case 2: positive camera offset (screen = world - cam, so screen < world)
cls()
camera(10,10)
pset(20,20,6)
local e=pget(20,20)      -- worldspace read while cam active -> should see 6
local f=pget(10,10)      -- rawscreen read while cam active -> should be background (0)
camera()
local g=pget(10,10)      -- rawscreen read after cam reset -> should see 6
local h=pget(20,20)      -- worldspace coord after cam reset -> background (0)

printh("RESULT pget_worldspace_while_cam_pos "..e)
printh("RESULT pget_rawscreen_while_cam_pos "..f)
printh("RESULT pget_rawscreen_after_reset_pos "..g)
printh("RESULT pget_worldspace_after_reset_pos "..h)

printh("DONE")
