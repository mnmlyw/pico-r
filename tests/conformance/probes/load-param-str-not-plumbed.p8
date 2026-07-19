pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- stat(6) is documented as the param_str last passed to run()/load(). This
-- probe uses run() (self-restart) as the vehicle rather than load()
-- (cross-cart warp) because the official binary's headless -x runner
-- never performs a cross-cart load() switch at all (confirmed: even
-- load()'ing the cart's own file back onto itself is a total no-op under
-- -x) -- so a load()-based probe can't observe the switch either side.
-- run() drives the identical switch/high-RAM-preserving machinery
-- (confirmed: RAM 0x8000+ survives it) and DOES switch under -x, making
-- the shared param_str/stat(6) channel observable.
--
-- stage, kept in persistent high RAM across each run() restart.
local stage = peek(0x8000)
if stage == 0 then
	printh("RESULT stat6_before_any_run "..stat(6))
	poke(0x8000, 1)
	run("alpha_param")
	printh("RESULT unreachable_stage0 yes")
elseif stage == 1 then
	printh("RESULT stat6_after_run_with_param "..stat(6))
	poke(0x8000, 2)
	run()
	printh("RESULT unreachable_stage1 yes")
else
	printh("RESULT stat6_after_run_no_param "..stat(6))
end
printh("DONE")
