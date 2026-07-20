pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- specials
printh("RESULT s_0_0 "..tostr(0^0,0x2))
printh("RESULT s_0_1 "..tostr(0^1,0x2))
printh("RESULT s_0_neg1 "..tostr(0^-1,0x2))
printh("RESULT s_0_half "..tostr(0^0.5,0x2))
printh("RESULT s_0_neghalf "..tostr(0^-0.5,0x2))
printh("RESULT s_1_big "..tostr(1^12345.678,0x2))
printh("RESULT s_1_negbig "..tostr(1^-12345.678,0x2))
printh("RESULT s_neg2_2 "..tostr((-2)^2,0x2))
printh("RESULT s_neg2_3 "..tostr((-2)^3,0x2))
printh("RESULT s_neg2_neg3 "..tostr((-2)^-3,0x2))
printh("RESULT s_neg2_half "..tostr((-2)^0.5,0x2))
printh("RESULT s_neg2_2p5 "..tostr((-2)^2.5,0x2))
printh("RESULT s_neg1p5_2 "..tostr((-1.5)^2,0x2))
printh("RESULT s_neg1p5_3 "..tostr((-1.5)^3,0x2))
printh("RESULT s_2_neghalf "..tostr(2^-0.5,0x2))
printh("RESULT s_9_neg1 "..tostr(9^-1,0x2))
printh("RESULT s_9_0p9 "..tostr(9^0.9,0x2))
printh("RESULT s_9_2 "..tostr(9^2,0x2))
printh("RESULT s_16_half "..tostr(16^0.5,0x2))
printh("RESULT s_9_0 "..tostr(9^0,0x2))
printh("RESULT s_big "..tostr(180^2,0x2))
printh("RESULT s_big2 "..tostr(2^15,0x2))
printh("RESULT s_big3 "..tostr(2^16,0x2))
printh("RESULT s_big4 "..tostr(2^17,0x2))
printh("RESULT s_2_neg17 "..tostr(2^-17,0x2))
printh("RESULT s_2_neg16 "..tostr(2^-16,0x2))
printh("RESULT s_3_7 "..tostr(3^7,0x2))
printh("RESULT s_1p5_2 "..tostr(1.5^2,0x2))
printh("RESULT s_1p1_3 "..tostr(1.1^3,0x2))
printh("RESULT mul_1p1_3 "..tostr(1.1*1.1*1.1,0x2))
printh("RESULT s_0p5_2 "..tostr(0.5^2,0x2))
printh("RESULT s_10_4 "..tostr(10^4,0x2))
printh("RESULT s_10_5 "..tostr(10^5,0x2))
-- x^1 for many x: is composition exact?
for k=1,40 do
 local x=k*0.7+0.05
 printh("RESULT one_"..k.." "..tostr(x,0x2).." "..tostr(x^1,0x2))
end
-- x^2 vs x*x
for k=1,20 do
 local x=k*0.61+0.13
 printh("RESULT sq_"..k.." "..tostr(x,0x2).." "..tostr(x^2,0x2).." "..tostr(x*x,0x2))
end
-- exp2 map coarse: 2^(k/256), k=0..256
for k=0,256 do
 printh("RESULT e"..k.." "..tostr(2^(k/256),0x2))
end
-- fine exp2 around 0: 2^(k/65536), k=0..32
for k=0,32 do
 printh("RESULT f"..k.." "..tostr(2^(k/65536),0x2))
end
printh("DONE")
