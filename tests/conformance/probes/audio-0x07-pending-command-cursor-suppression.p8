pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: \a (0x07) audio-annotation parse-state effect on print()'s
-- final cursor-y advance, across ~100 parameter-string shapes. The rule
-- (reverse-engineered from these + two held-out random validation
-- rounds against the official binary, all matching): the annotation is
-- scanned with a header phase (x/z/i/s/v = commands awaiting a numeric
-- argument; x/i/v take one digit OR one junk char and then exit the
-- header; z/s eat a multi-digit number, reprocess the char that ends
-- it, and never exit the header themselves) followed by a notes phase
-- (entered at the first note/junk char) where only x/i/v are commands.
-- print()'s final cursor advance is suppressed iff the string ENDS with
-- a command still awaiting its argument. Inputs that hang the official
-- binary (multi-digit command args like x12/s99, long bare-digit runs,
-- bare #) are deliberately excluded -- there is no golden-able behavior
-- for a hang.
cls() cursor(0,0) do local r=print("\aa") printh("RESULT a0 [a] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac") printh("RESULT a1 [c] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ag") printh("RESULT a2 [g] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ay") printh("RESULT a3 [y] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aq") printh("RESULT a4 [q] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\a1") printh("RESULT a5 [1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax") printh("RESULT a6 [x] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\az") printh("RESULT a7 [z] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ai") printh("RESULT a8 [i] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\as") printh("RESULT a9 [s] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\av") printh("RESULT a10 [v] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax1") printh("RESULT a11 [x1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax0") printh("RESULT a12 [x0] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ai5") printh("RESULT a13 [i5] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\as4") printh("RESULT a14 [s4] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\av3") printh("RESULT a15 [v3] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\az1") printh("RESULT a16 [z1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\as7q") printh("RESULT a17 [s7q] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\az1c") printh("RESULT a18 [z1c] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax3c1") printh("RESULT a19 [x3c1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\as4c1") printh("RESULT a20 [s4c1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ai5c1") printh("RESULT a21 [i5c1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\av3c1") printh("RESULT a22 [v3c1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\acx") printh("RESULT a23 [cx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aax") printh("RESULT a24 [ax] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\a1x") printh("RESULT a25 [1x] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ayx") printh("RESULT a26 [yx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aqx") printh("RESULT a27 [qx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aci") printh("RESULT a28 [ci] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\acv") printh("RESULT a29 [cv] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\a1i") printh("RESULT a30 [1i] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\a1v") printh("RESULT a31 [1v] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\a1z") printh("RESULT a32 [1z] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\a1s") printh("RESULT a33 [1s] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\a11v") printh("RESULT a34 [11v] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac1x") printh("RESULT a35 [c1x] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac1i") printh("RESULT a36 [c1i] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac1v") printh("RESULT a37 [c1v] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\accx") printh("RESULT a38 [ccx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac1cx") printh("RESULT a39 [c1cx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac#1x") printh("RESULT a40 [c#1x] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\acqx") printh("RESULT a41 [cqx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ayyx") printh("RESULT a42 [yyx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aqqx") printh("RESULT a43 [qqx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\acz") printh("RESULT a44 [cz] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\acs") printh("RESULT a45 [cs] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac1z") printh("RESULT a46 [c1z] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac1s") printh("RESULT a47 [c1s] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axys") printh("RESULT a48 [xys] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ayz") printh("RESULT a49 [yz] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aqz") printh("RESULT a50 [qz] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax3s") printh("RESULT a51 [x3s] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ai5z") printh("RESULT a52 [i5z] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ai5s") printh("RESULT a53 [i5s] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\av3z") printh("RESULT a54 [v3z] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax0z") printh("RESULT a55 [x0z] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axx") printh("RESULT a56 [xx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axc") printh("RESULT a57 [xc] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axa") printh("RESULT a58 [xa] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axi") printh("RESULT a59 [xi] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axq") printh("RESULT a60 [xq] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\asi") printh("RESULT a61 [si] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aix") printh("RESULT a62 [ix] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aiz") printh("RESULT a63 [iz] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\azz") printh("RESULT a64 [zz] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\azc") printh("RESULT a65 [zc] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\azzs") printh("RESULT a66 [zzs] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ascs") printh("RESULT a67 [scs] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\asvz") printh("RESULT a68 [svz] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\azzz") printh("RESULT a69 [zzz] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\azzx") printh("RESULT a70 [zzx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axyx") printh("RESULT a71 [xyx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axyi") printh("RESULT a72 [xyi] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axyz") printh("RESULT a73 [xyz] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\axyzw") printh("RESULT a74 [xyzw] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aizs") printh("RESULT a75 [izs] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aizx") printh("RESULT a76 [izx] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\asqzxi") printh("RESULT a77 [sqzxi] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\az9z") printh("RESULT a78 [z9z] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\as4z") printh("RESULT a79 [s4z] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\az9x") printh("RESULT a80 [z9x] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\az9s") printh("RESULT a81 [z9s] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\az9i") printh("RESULT a82 [z9i] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\az9zz") printh("RESULT a83 [z9zz] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\as4s4") printh("RESULT a84 [s4s4] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\az0z0") printh("RESULT a85 [z0z0] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax3x") printh("RESULT a86 [x3x] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax3i") printh("RESULT a87 [x3i] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax3v") printh("RESULT a88 [x3v] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ai5x") printh("RESULT a89 [i5x] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ai5i") printh("RESULT a90 [i5i] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax0x") printh("RESULT a91 [x0x] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ax1x1") printh("RESULT a92 [x1x1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac1x1") printh("RESULT a93 [c1x1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\acx1") printh("RESULT a94 [cx1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ai5i5x") printh("RESULT a95 [i5i5x] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac#") printh("RESULT a96 [c#] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac-1") printh("RESULT a97 [c-1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac1e1g1") printh("RESULT a98 [c1e1g1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\ac1e1g1c2") printh("RESULT a99 [c1e1g1c2] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\aegc") printh("RESULT a100 [egc] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\acc1") printh("RESULT a101 [cc1] r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("ab\ax") printh("RESULT comp1 r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("a\nb\ax") printh("RESULT comp2 r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("\^ta\ax") printh("RESULT comp3 r="..tostr(r).." cy="..peek(0x5f27)) end
cls() cursor(0,0) do local r=print("ab\acc") printh("RESULT comp4 r="..tostr(r).." cy="..peek(0x5f27)) end
printh("DONE")
