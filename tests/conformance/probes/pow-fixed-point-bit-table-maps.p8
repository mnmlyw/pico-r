pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- exp2 single-bit table: 2^(2^-k)
local b=0x0.8000
for k=1,16 do
 printh("RESULT bit"..k.." "..tostr(2^b,0x2))
 b=b/2
end
-- combos of bits
printh("RESULT c_c000 "..tostr(2^0x0.c000,0x2))
printh("RESULT c_ffff "..tostr(2^0x0.ffff,0x2))
printh("RESULT c_8001 "..tostr(2^0x0.8001,0x2))
printh("RESULT c_5555 "..tostr(2^0x0.5555,0x2))
printh("RESULT c_aaaa "..tostr(2^0x0.aaaa,0x2))
printh("RESULT c_0003 "..tostr(2^0x0.0003,0x2))
printh("RESULT c_000f "..tostr(2^0x0.000f,0x2))
printh("RESULT c_00ff "..tostr(2^0x0.00ff,0x2))
printh("RESULT c_0f0f "..tostr(2^0x0.0f0f,0x2))
printh("RESULT c_9000 "..tostr(2^0x0.9000,0x2))
printh("RESULT c_8800 "..tostr(2^0x0.8800,0x2))
printh("RESULT c_4400 "..tostr(2^0x0.4400,0x2))
-- 2^(1+f) vs 2*2^f decomposition
printh("RESULT d_1p5 "..tostr(2^1.5,0x2))
printh("RESULT d_0p5 "..tostr(2^0.5,0x2))
printh("RESULT d_2p5 "..tostr(2^2.5,0x2))
printh("RESULT d_10p5 "..tostr(2^10.5,0x2))
printh("RESULT d_9_2p5 "..tostr(9^2.5,0x2))
printh("RESULT d_9_0p5 "..tostr(9^0.5,0x2))
printh("RESULT d_9_2 "..tostr(9^2,0x2))
printh("RESULT d_prod "..tostr((9^2)*(9^0.5),0x2))
printh("RESULT d_5_3p7 "..tostr(5^3.7,0x2))
printh("RESULT d_5_0p7 "..tostr(5^0.7,0x2))
printh("RESULT d_5_3 "..tostr(5^3,0x2))
printh("RESULT d_prod2 "..tostr((5^3)*(5^0.7),0x2))
-- integer powering order: naive vs binary for x=1.1
local x=1.1
printh("RESULT i_x "..tostr(x,0x2))
printh("RESULT i_p4 "..tostr(x^4,0x2))
printh("RESULT i_naive4 "..tostr(((x*x)*x)*x,0x2))
printh("RESULT i_bin4 "..tostr((x*x)*(x*x),0x2))
printh("RESULT i_p5 "..tostr(x^5,0x2))
printh("RESULT i_naive5 "..tostr((((x*x)*x)*x)*x,0x2))
printh("RESULT i_bin5 "..tostr(((x*x)*(x*x))*x,0x2))
printh("RESULT i_p7 "..tostr(x^7,0x2))
printh("RESULT i_naive7 "..tostr(((((((x*x)*x)*x)*x)*x)*x),0x2))
local x2=x*x local x4=x2*x2
printh("RESULT i_bin7a "..tostr(x4*x2*x,0x2))
printh("RESULT i_bin7b "..tostr(x*x2*x4,0x2))
-- another base for order test
local w=0x0.0007
printh("RESULT w_p5 "..tostr(w^5,0x2))
printh("RESULT w_naive5 "..tostr((((w*w)*w)*w)*w,0x2))
local w2=w*w
printh("RESULT w_bin5 "..tostr((w2*w2)*w,0x2))
-- sub-1 bases fractional exponent
printh("RESULT u_p5_p5 "..tostr(0.5^0.5,0x2))
printh("RESULT u_p5_p25 "..tostr(0.5^0.25,0x2))
printh("RESULT u_p1_p5 "..tostr(0.1^0.5,0x2))
printh("RESULT u_p9_p3 "..tostr(0.9^0.3,0x2))
printh("RESULT u_p9_3p3 "..tostr(0.9^3.3,0x2))
-- log2 probing: x^0.5 for many x, and 4^f vs 2^(2f)
for k=0,64 do
 local f=k*0x0.04
 printh("RESULT g4_"..k.." "..tostr(4^f,0x2).." "..tostr(2^(2*f),0x2))
end
-- x^0.5 map
for k=1,80 do
 local x=k*0x0.28+0x0.01
 printh("RESULT h_"..k.." "..tostr(x,0x2).." "..tostr(x^0.5,0x2))
end
-- fine exp2 grid near 0 and near 1 (raw steps)
for k=0,64 do
 printh("RESULT fa"..k.." "..tostr(2^(k*0x0.0001),0x2))
end
for k=0,64 do
 printh("RESULT fb"..k.." "..tostr(2^(0x0.ff00+k*0x0.0001),0x2))
end
printh("DONE")
