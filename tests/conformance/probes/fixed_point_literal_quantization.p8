pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Literal quantization to the 16.16 grid (increment 1/65536 = 0.0000152588).
-- Determines whether the parser truncates or rounds-to-nearest, and
-- confirms sub-increment literals collapse to exact grid points (or zero).
printh("RESULT smallest_pos "..(0.0000152588))
printh("RESULT below_half_incr "..(0.00000762939))
printh("RESULT tiny_neg "..(-0.00001))
printh("RESULT tiny_pos "..(0.00001))
printh("RESULT near_incr_boundary_lo "..(0.99998))
printh("RESULT near_incr_boundary_hi "..(0.99999))
printh("RESULT round_trip_check "..(0.9999847412))
printh("RESULT round_up_candidate "..(0.99999237060546875))
printh("RESULT hex_frac "..(0x0.0001))
printh("RESULT hex_frac_half_incr "..(0x0.00008))
printh("RESULT hex_max "..(0x7fff.ffff))
printh("DONE")
