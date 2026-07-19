pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
printh("RESULT min_one_arg "..min(5))
printh("RESULT max_one_arg_neg "..max(-5))
printh("RESULT min_three_args "..min(5,10,1))
printh("RESULT max_three_args "..max(20,5,100))
printh("RESULT min_two_args "..min(5,10))
printh("RESULT max_two_args "..max(20,5))
printh("DONE")
