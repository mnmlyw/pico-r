pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- split() with a first argument that is neither a string nor a number
-- returns ZERO VALUES. Not nil, not {}, not {""} -- and it does NOT raise an
-- error; the cart carries on. select('#',...) is what tells "no values"
-- apart from "one nil value", which a plain tostr() cannot.
--
-- expected_failures.txt used to record this as official raising a runtime
-- error that aborts the cart. That was wrong, and a fix written to it would
-- have been fabricated while still passing CI.
printh("RESULT n_nil "..select('#', split(nil)))
printh("RESULT n_none "..select('#', split()))
printh("RESULT n_bool "..select('#', split(true)))
printh("RESULT n_tbl "..select('#', split({})))
printh("RESULT n_str "..select('#', split("a,b")))
printh("RESULT n_num "..select('#', split(12)))
-- numbers ARE accepted, via the same conversion tostr() uses
local r=split(12) printh("RESULT num_val "..type(r).." "..tostr(r[1]))
local q=split(1/3) printh("RESULT frac_val "..tostr(q[1]))
-- the sep/convert arguments do not change the gate
printh("RESULT n_nil_sep "..select('#', split(nil,"-")))
printh("RESULT n_nil_conv "..select('#', split(nil,",",false)))
printh("RESULT alive yes")
printh("DONE")
