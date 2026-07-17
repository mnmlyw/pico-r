pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Default number-to-string display: rounded to 4 decimal digits, trailing
-- zeros stripped (not Rust's shortest-round-trip float formatting).
printh("RESULT half "..(0.5))
printh("RESULT tenth "..(0.1))
printh("RESULT neg_tenth "..(-0.1))
printh("RESULT hundredth "..(0.01))
printh("RESULT big_frac "..(100.5))
printh("RESULT tiny "..(0.00001))
printh("RESULT neg_tiny "..(-0.00001))
printh("RESULT one "..(1))
printh("RESULT neg_one "..(-1))
printh("RESULT one_point_five "..(1.5))
printh("RESULT max_val "..(32767.99999))
printh("RESULT near_int "..(2.99998))
printh("RESULT quarter "..(0.25))
printh("RESULT one_third "..(1/3))
printh("DONE")
