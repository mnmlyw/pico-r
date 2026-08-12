use pico_r::{
    cart, gfx,
    memory::{self, Memory},
    palette, preprocessor,
    state::PicoState,
};

// Screen-palette register decode, captured off the real console via
// extcmd("screen") composited frames. Bit 7 selects the extended half of
// palette::COLORS, bits 0-3 index within it, bits 4-6 are ignored -- so this
// is neither a `& 0x0F` nor a `& 0x1F` mask, both of which get most of these
// wrong. (register value, expected palette::COLORS index)
const SCREEN_PAL_ORACLE: [(u8, usize); 8] = [
    (0x7e, 14),
    (0x10, 0),
    (0x8a, 26),
    (0xff, 31),
    (0x1e, 14),
    (0x0e, 14),
    (0x80, 16),
    (0x8f, 31),
];

#[test]
fn screen_palette_register_selects_extended_half_by_bit_7() {
    for (reg, expected_idx) in SCREEN_PAL_ORACLE {
        let mut memory = Memory::new();
        memory.init_draw_state();
        for y in 0..128u8 {
            for x in 0..128u8 {
                memory.screen_set(x, y, 1);
            }
        }
        memory.ram[memory::ADDR_SCREEN_PAL as usize + 1] = reg;

        let mut buf = [0u32; 128 * 128];
        gfx::render_to_argb(&memory, &mut buf);

        assert_eq!(
            buf[64 * 128 + 64],
            palette::COLORS[expected_idx],
            "screen-pal register {reg:#04x} should composite to COLORS[{expected_idx}]"
        );
    }
}

#[test]
fn p8_text_cart_loads_lua_and_data_sections() {
    let data = b"pico-8 cartridge // http://www.pico-8.com\n\
version 41\n\
__lua__\n\
x=1\n\
__gfx__\n\
0123456789abcdef\n\
__gff__\n\
ff00\n\
__map__\n\
2a00\n\
__sfx__\n\
00010203\n\
__music__\n\
01000000000\n";

    let mut state = PicoState::new();
    let cart = cart::load_bytes(data, &mut state.memory).expect("load text cart");

    assert_eq!(cart.lua_code, b"x=1\n");
    for x in 0..16 {
        assert_eq!(state.memory.sprite_get(x, 0), x);
    }
    assert_eq!(state.memory.ram[memory::ADDR_FLAGS as usize], 0xff);
    assert_eq!(state.memory.ram[memory::ADDR_FLAGS as usize + 1], 0x00);
    assert_eq!(state.memory.map_get(0, 0), 0x2a);
    assert_eq!(state.memory.ram[memory::ADDR_SFX as usize], 0x00);
    assert_eq!(state.memory.ram[memory::ADDR_SFX as usize + 1], 0x01);
    assert_eq!(state.memory.ram[memory::ADDR_SFX as usize + 2], 0x02);
    assert_eq!(state.memory.ram[memory::ADDR_SFX as usize + 3], 0x03);
    assert_eq!(state.memory.ram[memory::ADDR_MUSIC as usize] & 0x80, 0x80);
}

#[test]
fn preprocessor_keeps_pico8_dialect_transforms_stable() {
    assert_eq!(preprocessor::preprocess(b"a!=b"), b"a~=b");
    assert_eq!(preprocessor::preprocess(b"?a"), b"print(a)");
    assert_eq!(preprocessor::preprocess(b"x=0b101.1"), b"x=5.5");
}

#[test]
fn preprocessor_maps_high_byte_glyphs_to_p8_identifiers() {
    // 0x80 is a P8SCII glyph that's not a button shorthand — it passes
    // through raw (the lexer treats high bytes as identifier characters,
    // so a glyph variable's real name IS its raw byte, matching official
    // where `t={<glyph>=1}` and `t["<glyph>"]` are the same slot).
    let src: &[u8] = b"x=\x80";
    let out = preprocessor::preprocess(src);
    assert!(
        out.windows(1).any(|w| w == [0x80]),
        "expected raw glyph byte to pass through; got {out:?}"
    );
}

#[test]
fn preprocessor_bitwise_ops_bind_with_lua53_precedence() {
    // The preprocessor rewrites infix bitwise ops into calls as TEXT, so it
    // has to pick each operand's extent without a parser. These lock the
    // extents against the oracle-measured binding order (see the probe
    // bitwise-precedence-lua53-table): arithmetic and concat bind tighter
    // than any bitwise op, shifts tighter than `&`, `&` tighter than `^^`,
    // `^^` tighter than `|`, and comparisons looser than all of them.
    let cases: &[(&[u8], &str)] = &[
        // left operand spans a whole arithmetic expression, not just `b`
        (b"x=a+b>>1", "shr(a+b,1)"),
        // right operand keeps going through whitespace and arithmetic
        (b"x=1 >>> 16 - 8", "lshr(1,16 - 8)"),
        // `&` binds tighter than `|`, so it becomes the inner call
        (b"x=4|3&1", "bor(4,band(3,1))"),
        // same-level runs are left-associative
        (b"x=7&6&5", "band(band(7,6),5)"),
        // concat binds tighter, so it stays inside the operand
        (b"x=1 .. 2 & 3", "band(1 .. 2,3)"),
        // comparison binds looser, so it stays outside the call
        (b"x=dget(0)&1<<e>0", "band(dget(0),shl(1,e))>0"),
        // a call is one operand -- the extent must not stop inside its parens
        (b"x=f(a+b)>>1", "shr(f(a+b),1)"),
    ];
    for (src, expected) in cases {
        let out = preprocessor::preprocess(src);
        let out = String::from_utf8_lossy(&out);
        assert!(
            out.contains(expected),
            "{:?} should desugar to contain {expected:?}; got {out:?}",
            String::from_utf8_lossy(src)
        );
    }
}

#[test]
fn preprocessor_bitwise_operand_stops_at_term_boundaries() {
    // The left operand is bounded by whatever ended the previous term. Each
    // of these would silently swallow unrelated code if that tracking were
    // wrong -- and the generated call names are themselves a trap, since
    // `band(` contains `and` and `bor(` contains `or`.
    let cases: &[(&[u8], &str)] = &[
        (b"return a&1", "return band(a,1)"),
        (b"if a&1 then end", "if band(a,1) then end"),
        (b"a=b and c&1", "a=b and band(c,1)"),
        // two golfed-together statements: the operand ends at the space
        (b"x=1&2 y=3", "x=band(1,2) y=3"),
        // `end` here is a field name, not the keyword
        (b"x=t.end_x&1", "band(t.end_x,1)"),
        // a keyword inside a string must not end the operand
        (b"x=a&\"end\"", "band(a,\"end\")"),
    ];
    for (src, expected) in cases {
        let out = preprocessor::preprocess(src);
        let out = String::from_utf8_lossy(&out);
        assert!(
            out.contains(expected),
            "{:?} should desugar to contain {expected:?}; got {out:?}",
            String::from_utf8_lossy(src)
        );
    }
}

#[test]
fn preprocessor_skips_short_if_when_body_continues_with_or_and() {
    // Short-if expansion must NOT fire when the body looks like a multi-line
    // condition continuation. Otherwise `if (a) or` becomes `if a then or end`.
    let src: &[u8] = b"if (a) or\n(b) then end";
    let out = preprocessor::preprocess(src);
    let out = String::from_utf8_lossy(&out);
    assert!(
        !out.contains("then or"),
        "short-if should not fire on or-continuation; got {out:?}"
    );
}

#[test]
fn preprocessor_short_if_terminates_at_unmatched_end() {
    // Oracle-confirmed (see LEDGER + shortif-end-balance-rules probe): a
    // short-if body ends at the first `end` not matched by an opener
    // inside the body -- the synthetic `end` is flushed BEFORE it, and the
    // explicit `end` is left to close the enclosing block. Statements
    // after it on the same line parse OUTSIDE that block.
    let src: &[u8] = b"for i=1,3 do if(i==2)x+=1 end x+=100";
    let out = preprocessor::preprocess(src);
    let out = String::from_utf8_lossy(&out);
    let syn = out
        .find(" end end")
        .expect("synthetic end flushed before the explicit end");
    assert!(
        out[syn + 8..].contains("x = x + (100)") || out[syn + 8..].contains("x+=100"),
        "trailing statement must be after both ends; got {out:?}"
    );
}

#[test]
fn preprocessor_if_cond_do_is_block_if_not_short_if() {
    // Oracle-confirmed: `if(cond)do ... end` is a block-if -- the `do`
    // acts as the separator (rewritten to `then` for the parser) and the
    // body ends at its matching `end`; no synthetic end is owed.
    let src: &[u8] = b"if(x>0)do x+=1 end x+=10";
    let out = preprocessor::preprocess(src);
    let out = String::from_utf8_lossy(&out);
    assert!(
        out.contains("then"),
        "do separator rewritten to then; got {out:?}"
    );
    assert_eq!(
        out.matches("end").count(),
        1,
        "block-if owes no synthetic end; got {out:?}"
    );
}

#[test]
fn fixture_hello_p8_loads_clean() {
    let data = include_bytes!("fixtures/hello.p8");
    let mut state = PicoState::new();
    let cart = cart::load_bytes(data, &mut state.memory).expect("hello.p8 loads");
    assert!(String::from_utf8_lossy(&cart.lua_code).contains("function _init"));
}

#[test]
fn memory_page_registers_redirect_screen_and_sprite_access() {
    let mut memory = Memory::new();
    memory.init_draw_state();

    memory.screen_set(2, 3, 12);
    let screen_byte = memory.ram[memory::ADDR_SCREEN as usize + 3 * 64 + 1];
    assert_eq!(screen_byte & 0x0f, 12);

    memory.ram[memory::ADDR_SCREEN_PAGE as usize] = 0x00;
    memory.screen_set(0, 0, 7);
    assert_eq!(memory.ram[0] & 0x0f, 7);

    memory.ram[memory::ADDR_SCREEN as usize] = 0x0b;
    memory.ram[memory::ADDR_SPRITE_PAGE as usize] = 0x60;
    assert_eq!(memory.sprite_get(0, 0), 0x0b);
}
