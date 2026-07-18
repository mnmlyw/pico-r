pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- stat(16-19) is the per-channel sfx index, stat(20-23) is the note
-- index -- pico-r had these two ranges swapped.
sfx(3,0)
printh("RESULT stat16 "..stat(16))
printh("RESULT stat20 "..stat(20))

-- stat(100) (breadcrumb label) is nil when unset, not the generic 0
-- other unhandled codes fall back to.
printh("RESULT stat100_type "..type(stat(100)))

printh("DONE")
