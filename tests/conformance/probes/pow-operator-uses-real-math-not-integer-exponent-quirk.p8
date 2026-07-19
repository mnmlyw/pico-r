pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
srand(1)
printh("RESULT pow_frac "..(9^0.9))
printh("RESULT pow_half "..(16^0.5))
printh("RESULT pow_neg "..(9^-1))
printh("RESULT pow_int_control "..(9^2))
printh("RESULT pow_zero "..(9^0))
printh("RESULT pow_neg_frac "..(2^-0.5))
printh("DONE")
