pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- The persisted-progress idiom, lifted verbatim from a corpus cart
-- (to_take_root_among_the_stars_1-7): an unlock bitmask stored in cartdata
-- slot 0, set with `dset(0,dget(0)|1<<e)` and tested with `dget(0)&1<<e>0`.
--
-- Both lines are parse-extent traps. `dget(0)&1<<e>0` must bind as
-- `(dget(0) & (1<<e)) > 0`; binding the shift's left operand as `dget(0)&1`
-- instead makes every unlock test read the wrong bit, and the `|` version
-- writes a wrong byte that then PERSISTS to the save file. This is the
-- worst failure shape available -- silent, wrong, and durable -- so it is
-- worth locking separately from the abstract precedence table.
--
-- dset(0,0) first: cartdata persists across runs, so without an explicit
-- reset the golden would depend on whatever a previous run left behind.
cartdata("picor_bitwise_flag_roundtrip")
dset(0,0)

function unlocked(e) return dget(0)&1<<e>0 end
function unlock(e) dset(0,dget(0)|1<<e) end

unlock(2)
unlock(5)

printh("RESULT raw "..dget(0))
printh("RESULT bit0 "..tostr(unlocked(0)))
printh("RESULT bit2 "..tostr(unlocked(2)))
printh("RESULT bit5 "..tostr(unlocked(5)))
printh("RESULT bit7 "..tostr(unlocked(7)))

-- the same test written with the shift on the left of the mask
printh("RESULT shifted_mask "..tostr(1<<2&dget(0)>0))

printh("DONE")
