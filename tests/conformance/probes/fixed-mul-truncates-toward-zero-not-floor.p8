pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
printh("RESULT mul_neg_a "..tostr(-3.14159*2.71828,0x2))
printh("RESULT mul_neg_b "..tostr(-0.1*0.2,0x2))
printh("RESULT mul_neg_c "..tostr(-12.34*5.67,0x2))
printh("RESULT mul_pos_control "..tostr(3.14159*2.71828,0x2))
printh("RESULT mul_friction_1 "..tostr(-1*0.85,0x2))
printh("RESULT mul_friction_2 "..tostr(-0.0001*0.85,0x2))
printh("RESULT mul_both_neg "..tostr(-3.14159*-2.71828,0x2))
printh("DONE")
