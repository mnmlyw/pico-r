// Standard-library subset + PICO-8 API registration.
//
// Native functions take (&mut Interp, Vec<Value>) -> Result<Vec<Value>, RtError>.
// Anything that touches PICO-8 state goes through `interp.host()`.

use std::cell::RefCell;
use std::rc::Rc;

use super::interp::Interp;
use super::value::*;

use crate::gfx;
use crate::memory as mem;
use crate::state::PicoState;

// === Helpers ===

fn rc_str(s: &str) -> Rc<[u8]> {
    Rc::from(s.as_bytes())
}
fn nil() -> Value {
    Value::Nil
}
fn num(n: f64) -> Value {
    Value::Number(n)
}
fn boolv(b: bool) -> Value {
    Value::Bool(b)
}
fn str_v(b: &[u8]) -> Value {
    Value::Str(Rc::from(b))
}

fn arg_num(args: &[Value], i: usize) -> Option<f64> {
    args.get(i).and_then(|v| v.as_number())
}
fn arg_int(args: &[Value], i: usize) -> Option<i32> {
    arg_num(args, i).map(safe_to_i32)
}
fn opt_int(args: &[Value], i: usize, d: i32) -> i32 {
    if i >= args.len() || matches!(args[i], Value::Nil) {
        d
    } else {
        arg_int(args, i).unwrap_or(d)
    }
}
fn opt_num(args: &[Value], i: usize, d: f64) -> f64 {
    if i >= args.len() || matches!(args[i], Value::Nil) {
        d
    } else {
        arg_num(args, i).unwrap_or(d)
    }
}
fn opt_bool(args: &[Value], i: usize, d: bool) -> bool {
    if i >= args.len() || matches!(args[i], Value::Nil) {
        d
    } else {
        args[i].truthy()
    }
}
fn arg_str(args: &[Value], i: usize) -> Option<Rc<[u8]>> {
    args.get(i).and_then(|v| v.as_str())
}

fn safe_to_i32(f: f64) -> i32 {
    if f.is_nan() {
        return 0;
    }
    if f >= i32::MAX as f64 {
        return i32::MAX;
    }
    if f <= i32::MIN as f64 {
        return i32::MIN;
    }
    f as i32
}

// Memory addresses use flr() semantics (floor toward -infinity), not
// truncate-toward-zero: confirmed against official PICO-8 that
// peek(-0.5) reads the same byte as peek(-1) (both wrap to 0xffff), not
// address 0 (which is what `as i32` truncation would give for -0.5).
// Scoped to address arguments specifically -- not applied to arg_int
// generally, since that's used pervasively for unrelated arguments this
// hasn't been verified for.
fn arg_addr(args: &[Value], i: usize) -> u16 {
    let f = arg_num(args, i).unwrap_or(0.0);
    if f.is_nan() {
        return 0;
    }
    safe_to_i32(f.floor()) as u16
}

fn to_fixed(v: f64) -> i32 {
    let scaled = v * 65536.0;
    if scaled.is_nan() {
        return 0;
    }
    if scaled > i32::MAX as f64 || scaled < i32::MIN as f64 {
        let wide = scaled as i64;
        return wide as i32;
    }
    scaled as i32
}
fn from_fixed(v: i32) -> f64 {
    v as f64 / 65536.0
}

// === Color helper for gfx ===

fn get_color(state: &mut PicoState, args: &[Value], i: usize) -> u8 {
    if i >= args.len() || matches!(args[i], Value::Nil) {
        return state.memory.ram[mem::ADDR_COLOR as usize] & 0x0F;
    }
    let c = (arg_int(args, i).unwrap_or(0) as u32 & 0xFF) as u8;
    state.memory.ram[mem::ADDR_COLOR as usize] = c;
    c & 0x0F
}

// === Native function registration ===

pub fn register_all(globals: &Rc<Table>) {
    let mut g = globals.borrow_mut();
    let mut set =
        |name: &'static str, f: fn(&mut Interp, Vec<Value>) -> Result<Vec<Value>, RtError>| {
            let nf = Rc::new(NativeFn {
                name,
                func: Box::new(f),
            });
            g.set(
                Value::Str(rc_str(name)),
                Value::Function(Function::Native(nf)),
            );
        };

    // === stdlib ===
    set("type", lua_type);
    set("tostring", lua_tostring);
    set("tonumber", lua_tonumber);
    set("pairs", lua_pairs);
    set("ipairs", lua_ipairs);
    set("next", lua_next);
    set("inext", lua_inext);
    set("select", lua_select);
    set("setmetatable", lua_setmetatable);
    set("getmetatable", lua_getmetatable);
    set("rawget", lua_rawget);
    set("rawset", lua_rawset);
    set("rawequal", lua_rawequal);
    set("rawlen", lua_rawlen);
    set("assert", lua_assert);

    // === Math ===
    set("abs", api_abs);
    set("flr", api_flr);
    set("ceil", api_ceil);
    set("sqrt", api_sqrt);
    set("sin", api_sin);
    set("cos", api_cos);
    set("atan2", api_atan2);
    set("max", api_max);
    set("min", api_min);
    set("mid", api_mid);
    set("rnd", api_rnd);
    set("srand", api_srand);
    set("sgn", api_sgn);

    // === Bitwise ===
    set("band", api_band);
    set("bor", api_bor);
    set("bxor", api_bxor);
    set("bnot", api_bnot);
    set("shl", api_shl);
    set("shr", api_shr);
    set("lshr", api_lshr);
    set("rotl", api_rotl);
    set("rotr", api_rotr);

    // === Strings ===
    set("tostr", api_tostr);
    set("tonum", api_tonum);
    set("sub", api_sub);
    set("chr", api_chr);
    set("ord", api_ord);
    set("split", api_split);

    // === Table ===
    set("add", api_add);
    set("del", api_del);
    set("deli", api_deli);
    set("count", api_count);
    set("foreach", api_foreach);
    set("all", api_all);
    set("pack", api_pack);
    set("unpack", api_unpack);

    // === Graphics ===
    set("cls", api_cls);
    set("pset", api_pset);
    set("pget", api_pget);
    set("line", api_line);
    set("tline", api_tline);
    set("rect", api_rect);
    set("rectfill", api_rectfill);
    set("circ", api_circ);
    set("circfill", api_circfill);
    set("oval", api_oval);
    set("ovalfill", api_ovalfill);
    set("spr", api_spr);
    set("sspr", api_sspr);
    set("map", api_map);
    set("mget", api_mget);
    set("mset", api_mset);
    set("sget", api_sget);
    set("sset", api_sset);
    set("fget", api_fget);
    set("fset", api_fset);
    set("print", api_print);
    set("cursor", api_cursor);
    set("color", api_color);
    set("camera", api_camera);
    set("clip", api_clip);
    set("pal", api_pal);
    set("palt", api_palt);
    set("fillp", api_fillp);

    // === Input ===
    set("btn", api_btn);
    set("btnp", api_btnp);

    // === Memory ===
    set("peek", api_peek);
    set("poke", api_poke);
    set("peek2", api_peek2);
    set("poke2", api_poke2);
    set("peek4", api_peek4);
    set("poke4", api_poke4);
    set("memcpy", api_memcpy);
    set("memset", api_memset);
    set("reload", api_reload);
    set("load", api_load);
    set("cstore", api_cstore);

    // === Audio ===
    set("sfx", api_sfx);
    set("music", api_music);

    // === System ===
    set("stat", api_stat);
    set("time", api_time);
    set("t", api_time);
    set("printh", api_printh);
    set("cartdata", api_cartdata);
    set("dget", api_dget);
    set("dset", api_dset);
    set("cocreate", api_cocreate);
    set("coresume", api_coresume);
    set("costatus", api_costatus);
    set("yield", api_yield);
    set("menuitem", api_menuitem);
    set("extcmd", api_extcmd);
    set("flip", api_flip);
    set("_set_fps", api_set_fps);
    set("serial", api_serial);
    set("readrom", api_readrom);
    set("reset", api_reset);
    set("stop", api_stop);
}

// === stdlib bodies ===

fn lua_type(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if args.is_empty() {
        return Ok(vec![]);
    }
    Ok(vec![str_v(args[0].type_name().as_bytes())])
}

fn lua_tostring(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = args.into_iter().next().unwrap_or(Value::Nil);
    let s = match &v {
        Value::Nil => "nil".to_string(),
        Value::Bool(b) => {
            if *b {
                "true".into()
            } else {
                "false".into()
            }
        }
        Value::Number(n) => number_to_str(*n),
        Value::Str(b) => return Ok(vec![Value::Str(Rc::clone(b))]),
        Value::Table(t) => format!("table: 0x{:x}", Rc::as_ptr(t) as usize),
        Value::Function(f) => format!("function: 0x{:x}", f.identity()),
    };
    Ok(vec![str_v(s.as_bytes())])
}

fn lua_tonumber(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = args.first().cloned().unwrap_or(Value::Nil);
    Ok(vec![match v.as_number() {
        Some(n) => num(n),
        None => nil(),
    }])
}

fn lua_pairs(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = args.into_iter().next().unwrap_or(Value::Nil);
    if matches!(t, Value::Nil) {
        // PICO-8: pairs(nil) returns empty iterator
        return Ok(vec![Value::Function(Function::Native(Rc::new(NativeFn {
            name: "empty_iter",
            func: Box::new(|_i, _a| Ok(vec![nil()])),
        })))]);
    }
    let next_fn = Value::Function(Function::Native(Rc::new(NativeFn {
        name: "next",
        func: Box::new(lua_next),
    })));
    Ok(vec![next_fn, t, nil()])
}

fn lua_ipairs(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = args.into_iter().next().unwrap_or(Value::Nil);
    if matches!(t, Value::Nil) {
        return Ok(vec![Value::Function(Function::Native(Rc::new(NativeFn {
            name: "empty_iter",
            func: Box::new(|_i, _a| Ok(vec![nil()])),
        })))]);
    }
    let iter = Value::Function(Function::Native(Rc::new(NativeFn {
        name: "ipairs_iter",
        func: Box::new(|_i, args| {
            let t = args.first().cloned().unwrap_or(Value::Nil);
            let i = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as i64;
            let next = i + 1;
            if let Value::Table(tbl) = &t {
                let v = tbl.borrow().get(&num(next as f64));
                if matches!(v, Value::Nil) {
                    return Ok(vec![nil()]);
                }
                return Ok(vec![num(next as f64), v]);
            }
            Ok(vec![nil()])
        }),
    })));
    Ok(vec![iter, t, num(0.0)])
}

fn lua_next(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = match args.first() {
        Some(Value::Table(t)) => t.clone(),
        _ => return Err(RtError::msg("next: not a table")),
    };
    let prev_key = args.get(1).cloned().unwrap_or(Value::Nil);
    let table = t.borrow();
    // Build a deterministic-ish ordering based on hashmap iteration. Lua doesn't
    // guarantee ordering anyway.
    let keys: Vec<Key> = table.map.keys().cloned().collect();
    let prev_keyed = Key::from_value(&prev_key);
    let mut found = matches!(prev_key, Value::Nil);
    for k in &keys {
        if found {
            let v = table.map.get(k).cloned().unwrap_or(Value::Nil);
            return Ok(vec![key_to_value(k), v]);
        }
        if let Some(pk) = &prev_keyed {
            if k == pk {
                found = true;
            }
        }
    }
    Ok(vec![nil()])
}

// `inext(t, i)` -- the sequence-indexed companion to next() (what
// `for i,v in inext,t` iterates with): returns i+1, t[i+1] until the
// first nil slot. Confirmed a real builtin via oracle; used by real
// corpus carts (kalikan_stage_1b-3.p8.png, redash-7.p8.png).
fn lua_inext(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = match args.first() {
        Some(Value::Table(t)) => t.clone(),
        _ => return Err(RtError::msg("inext: not a table")),
    };
    let prev = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n as i64)
        .unwrap_or(0);
    let next_i = prev + 1;
    let v = t.borrow().get(&Value::Number(next_i as f64));
    if matches!(v, Value::Nil) {
        return Ok(vec![nil()]);
    }
    Ok(vec![num(next_i as f64), v])
}

fn key_to_value(k: &Key) -> Value {
    match k {
        Key::Bool(b) => Value::Bool(*b),
        Key::Int(i) => Value::Number(*i as f64),
        Key::Float(b) => Value::Number(f64::from_bits(*b)),
        Key::Str(s) => Value::Str(Rc::clone(s)),
        Key::Table(_) | Key::Function(_) => Value::Nil, // not preserved in this impl
    }
}

fn lua_select(_i: &mut Interp, mut args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if args.is_empty() {
        return Err(RtError::msg("select: missing args"));
    }
    let first = args.remove(0);
    if let Value::Str(s) = &first {
        if &**s == b"#" {
            return Ok(vec![num(args.len() as f64)]);
        }
    }
    let i = first
        .as_number()
        .ok_or_else(|| RtError::msg("select: bad index"))? as i64;
    let n = args.len() as i64;
    let start = if i < 0 {
        (n + i).max(0)
    } else {
        (i - 1).max(0)
    };
    Ok(args.into_iter().skip(start as usize).collect())
}

fn lua_setmetatable(_i: &mut Interp, mut args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if args.len() < 2 {
        return Err(RtError::msg("setmetatable: 2 args"));
    }
    let mt = args.pop().unwrap();
    let t = args.pop().unwrap();
    if let Value::Table(tbl) = &t {
        let m = match mt {
            Value::Nil => None,
            Value::Table(m) => Some(m),
            _ => return Err(RtError::msg("setmetatable: not a table")),
        };
        tbl.borrow_mut().metatable = m;
        Ok(vec![t])
    } else {
        Err(RtError::msg("setmetatable: not a table"))
    }
}

fn lua_getmetatable(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let Some(Value::Table(t)) = args.first() {
        if let Some(mt) = &t.borrow().metatable {
            return Ok(vec![Value::Table(Rc::clone(mt))]);
        }
    }
    Ok(vec![nil()])
}

fn lua_rawget(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let (Some(Value::Table(t)), Some(k)) = (args.first(), args.get(1)) {
        return Ok(vec![t.borrow().get(k)]);
    }
    Ok(vec![nil()])
}

fn lua_rawset(_i: &mut Interp, mut args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if args.len() < 3 {
        return Err(RtError::msg("rawset: 3 args"));
    }
    let v = args.pop().unwrap();
    let k = args.pop().unwrap();
    let t = args.pop().unwrap();
    if let Value::Table(tbl) = &t {
        tbl.borrow_mut().set(k, v);
        return Ok(vec![t]);
    }
    Err(RtError::msg("rawset: not a table"))
}

fn lua_rawequal(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let a = args.first().cloned().unwrap_or(Value::Nil);
    let b = args.get(1).cloned().unwrap_or(Value::Nil);
    Ok(vec![boolv(a.raw_equal(&b))])
}

fn lua_rawlen(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    match args.first() {
        Some(Value::Str(s)) => Ok(vec![num(s.len() as f64)]),
        Some(Value::Table(t)) => Ok(vec![num(t.borrow().raw_len() as f64)]),
        _ => Err(RtError::msg("rawlen: needs table or string")),
    }
}

fn lua_assert(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = args.first().cloned().unwrap_or(Value::Nil);
    if !v.truthy() {
        let msg = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Str(rc_str("assertion failed!")));
        return Err(RtError { value: msg });
    }
    Ok(args)
}

// === Math ===

fn api_abs(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(arg_num(&a, 0).unwrap_or(0.0).abs())])
}
fn api_flr(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(arg_num(&a, 0).unwrap_or(0.0).floor())])
}
fn api_ceil(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(arg_num(&a, 0).unwrap_or(0.0).ceil())])
}
fn api_sqrt(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = arg_num(&a, 0).unwrap_or(0.0);
    Ok(vec![num(if v >= 0.0 { v.sqrt() } else { 0.0 })])
}
fn api_sin(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = arg_num(&a, 0).unwrap_or(0.0);
    Ok(vec![num(-(v * std::f64::consts::TAU).sin())])
}
fn api_cos(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = arg_num(&a, 0).unwrap_or(0.0);
    Ok(vec![num((v * std::f64::consts::TAU).cos())])
}
fn api_atan2(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let dx = arg_num(&a, 0).unwrap_or(0.0);
    let dy = arg_num(&a, 1).unwrap_or(0.0);
    if dx == 0.0 && dy == 0.0 {
        return Ok(vec![num(0.25)]);
    }
    // PICO-8 atan2 returns turns in [0,1) with y inverted
    let r = (-dy).atan2(dx);
    let mut t = r / std::f64::consts::TAU;
    if t < 0.0 {
        t += 1.0;
    }
    Ok(vec![num(t)])
}
fn api_max(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let mut r = arg_num(&a, 0).unwrap_or(0.0);
    for v in a.iter().skip(1) {
        r = r.max(v.as_number().unwrap_or(0.0));
    }
    Ok(vec![num(r)])
}
fn api_min(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let mut r = arg_num(&a, 0).unwrap_or(0.0);
    for v in a.iter().skip(1) {
        r = r.min(v.as_number().unwrap_or(0.0));
    }
    Ok(vec![num(r)])
}
fn api_mid(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = arg_num(&a, 0).unwrap_or(0.0);
    let y = arg_num(&a, 1).unwrap_or(0.0);
    let z = arg_num(&a, 2).unwrap_or(0.0);
    Ok(vec![num(x.min(y).max(x.max(y).min(z)))])
}
fn api_rnd(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let Some(Value::Table(t)) = a.first() {
        let len = t.borrow().raw_len();
        if len == 0 {
            return Ok(vec![nil()]);
        }
        let r = xorshift(i) % len as u32;
        let v = t.borrow().get(&num((r + 1) as f64));
        return Ok(vec![v]);
    }
    let max = arg_num(&a, 0).unwrap_or(1.0);
    let r = xorshift(i) as f64 / 4294967296.0;
    Ok(vec![num(r * max)])
}
fn api_srand(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = arg_num(&a, 0).unwrap_or(0.0);
    let s = to_fixed(v) as u32;
    i.host().rng_state = if s == 0 { 1 } else { s };
    Ok(vec![])
}
fn xorshift(i: &mut Interp) -> u32 {
    let host = i.host();
    if host.rng_state == 0 {
        host.rng_state = 0x12345678;
    }
    let mut s = host.rng_state;
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    host.rng_state = s;
    s
}
fn api_sgn(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = arg_num(&a, 0).unwrap_or(0.0);
    Ok(vec![num(if v < 0.0 { -1.0 } else { 1.0 })])
}

// === Bitwise (16:16 fixed) ===

fn api_band(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(from_fixed(
        to_fixed(arg_num(&a, 0).unwrap_or(0.0)) & to_fixed(arg_num(&a, 1).unwrap_or(0.0)),
    ))])
}
fn api_bor(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(from_fixed(
        to_fixed(arg_num(&a, 0).unwrap_or(0.0)) | to_fixed(arg_num(&a, 1).unwrap_or(0.0)),
    ))])
}
fn api_bxor(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(from_fixed(
        to_fixed(arg_num(&a, 0).unwrap_or(0.0)) ^ to_fixed(arg_num(&a, 1).unwrap_or(0.0)),
    ))])
}
fn api_bnot(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(from_fixed(!to_fixed(
        arg_num(&a, 0).unwrap_or(0.0),
    )))])
}
fn api_shl(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = to_fixed(arg_num(&a, 0).unwrap_or(0.0));
    let b = arg_int(&a, 1).unwrap_or(0);
    let r = if b >= 0 {
        x.wrapping_shl((b.min(31)) as u32)
    } else {
        x.wrapping_shr(((-b).min(31)) as u32)
    };
    Ok(vec![num(from_fixed(r))])
}
fn api_shr(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = to_fixed(arg_num(&a, 0).unwrap_or(0.0));
    let b = arg_int(&a, 1).unwrap_or(0);
    let r = if b >= 0 {
        x.wrapping_shr((b.min(31)) as u32)
    } else {
        x.wrapping_shl(((-b).min(31)) as u32)
    };
    Ok(vec![num(from_fixed(r))])
}
fn api_lshr(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = to_fixed(arg_num(&a, 0).unwrap_or(0.0)) as u32;
    let b = arg_int(&a, 1).unwrap_or(0).clamp(0, 31) as u32;
    Ok(vec![num(from_fixed((x >> b) as i32))])
}
fn api_rotl(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = to_fixed(arg_num(&a, 0).unwrap_or(0.0)) as u32;
    let b = (arg_int(&a, 1).unwrap_or(0).rem_euclid(32)) as u32;
    Ok(vec![num(from_fixed(x.rotate_left(b) as i32))])
}
fn api_rotr(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = to_fixed(arg_num(&a, 0).unwrap_or(0.0)) as u32;
    let b = (arg_int(&a, 1).unwrap_or(0).rem_euclid(32)) as u32;
    Ok(vec![num(from_fixed(x.rotate_right(b) as i32))])
}

// === String ===

fn display_string(v: &Value) -> String {
    match v {
        Value::Nil => "[nil]".to_string(),
        Value::Bool(b) => {
            if *b {
                "true".into()
            } else {
                "false".into()
            }
        }
        Value::Number(n) => number_to_str(*n),
        Value::Str(b) => String::from_utf8_lossy(b).into_owned(),
        Value::Table(_) => "[table]".to_string(),
        Value::Function(_) => "[function]".to_string(),
    }
}

fn api_tostr(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    // tostr() with truly no arguments is "" -- distinct from tostr(nil),
    // which is "[nil]" (confirmed against official PICO-8).
    if a.is_empty() {
        return Ok(vec![str_v(b"")]);
    }
    let v = a[0].clone();
    let flags = arg_int(&a, 1).unwrap_or(0);
    if flags != 0 {
        // Format flags only apply to numbers; confirmed against official
        // PICO-8: 0x1 = hex "0xHHHH.LLLL", 0x2 = raw 32-bit fixed value as a
        // plain decimal integer, 0x3 = hex digits with no "." separator.
        if let Value::Number(n) = v {
            let raw = to_fixed(n);
            let hex = flags & 0x1 != 0;
            let raw_dec = flags & 0x2 != 0;
            if hex && raw_dec {
                return Ok(vec![str_v(format!("0x{:08x}", raw as u32).as_bytes())]);
            } else if hex {
                let hi = (raw as u32) >> 16;
                let lo = (raw as u32) & 0xffff;
                return Ok(vec![str_v(format!("0x{:04x}.{:04x}", hi, lo).as_bytes())]);
            } else if raw_dec {
                return Ok(vec![str_v(format!("{}", raw).as_bytes())]);
            }
        }
    }
    if let Value::Str(b) = &v {
        return Ok(vec![Value::Str(Rc::clone(b))]);
    }
    Ok(vec![str_v(display_string(&v).as_bytes())])
}

fn api_tonum(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = a.first().cloned().unwrap_or(Value::Nil);
    let flags = arg_int(&a, 1).unwrap_or(0);
    // Confirmed against official PICO-8: 0x4 returns 0 instead of nil on a
    // parse failure.
    let fail = |flags: i32| if flags & 0x4 != 0 { num(0.0) } else { nil() };

    if flags & 0x1 != 0 {
        // Hex parse: optional "0x"/"0X" prefix, optional "." fractional
        // part (the inverse of tostr's 0x1 hex format).
        let Some(s) = v.as_str() else {
            return Ok(vec![fail(flags)]);
        };
        let s = String::from_utf8_lossy(&s);
        let s = s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(&s);
        let (int_part, frac_part) = match s.split_once('.') {
            Some((i, f)) => (i, Some(f)),
            None => (s, None),
        };
        if int_part.is_empty() && frac_part.is_none_or(|f| f.is_empty()) {
            return Ok(vec![fail(flags)]);
        }
        let hi = i64::from_str_radix(int_part, 16).unwrap_or(0);
        let lo = match frac_part {
            Some(f) if !f.is_empty() => i64::from_str_radix(f, 16)
                .map(|v| v as f64 / 16f64.powi(f.len() as i32))
                .unwrap_or(0.0),
            _ => 0.0,
        };
        return Ok(vec![num(hi as f64 + lo)]);
    }

    if flags & 0x2 != 0 {
        // Raw fixed-point reinterpretation (the inverse of tostr's 0x2):
        // a string is parsed as a plain integer raw bit pattern; a number
        // uses its own to_fixed() representation (which naturally wraps
        // for out-of-16.16-range values).
        let raw = match &v {
            Value::Str(s) => String::from_utf8_lossy(s).parse::<i64>().unwrap_or(0) as i32,
            Value::Number(n) => to_fixed(*n),
            _ => return Ok(vec![fail(flags)]),
        };
        return Ok(vec![num(from_fixed(raw))]);
    }

    // Booleans convert: tonum(true)==1, tonum(false)==0 -- confirmed
    // against official PICO-8 (carts use `tonum(btn"1")-tonum(btn"0")`
    // for directional input, e.g. deepening-0.p8.png).
    if let Value::Bool(b) = v {
        return Ok(vec![num(if b { 1.0 } else { 0.0 })]);
    }

    Ok(vec![match v.as_number() {
        Some(n) => num(n),
        None => fail(flags),
    }])
}

fn api_sub(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let s = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    let len = s.len() as i64;
    let start_raw = opt_int(&a, 1, 1) as i64;
    let end_raw = opt_int(&a, 2, len as i32) as i64;
    let start = if start_raw < 0 {
        (len + start_raw + 1).max(1)
    } else {
        start_raw.max(1)
    };
    let end = if end_raw < 0 {
        len + end_raw + 1
    } else {
        end_raw.min(len)
    };
    if start > end || start > len {
        return Ok(vec![str_v(b"")]);
    }
    let si = (start - 1) as usize;
    let ei = end as usize;
    Ok(vec![Value::Str(Rc::from(&s[si..ei]))])
}

fn api_chr(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let mut out = Vec::with_capacity(a.len());
    for v in &a {
        if let Some(n) = v.as_number() {
            // Confirmed against official PICO-8: out-of-range ordinals wrap
            // via `% 256` rather than being dropped (chr(256,300,-1) -> the
            // 3-byte string [0, 44, 255]).
            let n = (n as i32).rem_euclid(256);
            out.push(n as u8);
        }
    }
    Ok(vec![Value::Str(Rc::from(out.as_slice()))])
}

fn api_ord(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let s = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    let idx = opt_int(&a, 1, 1);
    let n_results = opt_int(&a, 2, 1);
    let mut out = Vec::new();
    for i in 0..n_results {
        let pos = idx + i;
        if pos >= 1 && (pos as usize) <= s.len() {
            out.push(num(s[(pos - 1) as usize] as f64));
        } else {
            out.push(nil());
        }
    }
    Ok(out)
}

// split()'s token->number conversion matches official PICO-8's tonum-like
// behavior, which also accepts hex (`0x6000`, with optional fraction) and
// binary (`0b11`) literals -- confirmed via oracle (split"0x10" yields the
// NUMBER 16, and split"0b11" yields 3; real corpus cart mer_ork-0.p8.png
// passes memory addresses around as split"0x6000,0xe000,0x1fff").
fn split_token_to_number(part: &[u8]) -> Option<f64> {
    // Tokens numberize with surrounding whitespace ignored (" 6" -> 6,
    // " 0x10" -> 16, oracle-confirmed) -- but a token that FAILS to
    // convert keeps its original spacing as a string, so only this
    // parsing path trims.
    let txt = std::str::from_utf8(part).ok()?.trim();
    if let Ok(f) = txt.parse::<f64>() {
        // Rust accepts "inf"/"nan"/"infinity"; PICO-8 does not.
        if txt.bytes().any(|b| b.is_ascii_digit()) {
            return Some(f);
        }
        return None;
    }
    let (neg, rest) = match txt.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, txt),
    };
    let radix_val = if let Some(h) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
        parse_radix_fraction(h, 16)
    } else if let Some(b) = rest.strip_prefix("0b").or_else(|| rest.strip_prefix("0B")) {
        parse_radix_fraction(b, 2)
    } else {
        None
    }?;
    Some(if neg { -radix_val } else { radix_val })
}

fn parse_radix_fraction(s: &str, radix: u32) -> Option<f64> {
    let (int_part, frac_part) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let mut v: f64 = 0.0;
    for c in int_part.chars() {
        v = v * radix as f64 + c.to_digit(radix)? as f64;
    }
    let mut scale = 1.0 / radix as f64;
    for c in frac_part.chars() {
        v += c.to_digit(radix)? as f64 * scale;
        scale /= radix as f64;
    }
    Some(v)
}

fn api_split(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let s = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    let convert = if a.len() <= 2 || matches!(a.get(2), Some(Value::Nil)) {
        true
    } else {
        a[2].truthy()
    };
    let t = Rc::new(RefCell::new(TableInner::new()));

    if let Some(Value::Number(n)) = a.get(1) {
        let n = if n.is_nan() || *n < 1.0 {
            1usize
        } else if *n > 255.0 {
            255
        } else {
            *n as usize
        };
        let mut idx: i64 = 1;
        let mut pos = 0usize;
        while pos < s.len() {
            let end = (pos + n).min(s.len());
            let part = &s[pos..end];
            let v = if convert {
                match split_token_to_number(part) {
                    Some(f) => num(f),
                    None => Value::Str(Rc::from(part)),
                }
            } else {
                Value::Str(Rc::from(part))
            };
            t.borrow_mut().set(num(idx as f64), v);
            idx += 1;
            pos = end;
        }
        return Ok(vec![Value::Table(t)]);
    }

    let sep = match a.get(1) {
        Some(v) => v.as_str().unwrap_or_else(|| Rc::from(b",".as_slice())),
        None => Rc::from(b",".as_slice()),
    };
    if sep.is_empty() {
        for (idx, &c) in (1_i64..).zip(s.iter()) {
            t.borrow_mut()
                .set(num(idx as f64), Value::Str(Rc::from(&[c][..])));
        }
        return Ok(vec![Value::Table(t)]);
    }
    let mut idx: i64 = 1;
    let mut start = 0usize;
    let mut i = 0usize;
    while i + sep.len() <= s.len() {
        if s[i..i + sep.len()] == sep[..] {
            let part = &s[start..i];
            let v = if convert {
                match split_token_to_number(part) {
                    Some(f) => num(f),
                    None => Value::Str(Rc::from(part)),
                }
            } else {
                Value::Str(Rc::from(part))
            };
            t.borrow_mut().set(num(idx as f64), v);
            idx += 1;
            i += sep.len();
            start = i;
        } else {
            i += 1;
        }
    }
    let part = &s[start..];
    let v = if convert {
        match split_token_to_number(part) {
            Some(f) => num(f),
            None => Value::Str(Rc::from(part)),
        }
    } else {
        Value::Str(Rc::from(part))
    };
    t.borrow_mut().set(num(idx as f64), v);
    Ok(vec![Value::Table(t)])
}

// === Table ===

fn api_add(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let Some(Value::Table(t)) = args.first() {
        let v = args.get(1).cloned().unwrap_or(Value::Nil);
        let pos = args.get(2).and_then(|x| x.as_number()).map(|n| n as i64);
        let mut tb = t.borrow_mut();
        let len = tb.raw_len();
        if let Some(p) = pos {
            if p < 1 {
                return Ok(vec![v]);
            }
            for k in (p..=len).rev() {
                let v_ = tb.map.get(&Key::Int(k)).cloned().unwrap_or(Value::Nil);
                tb.map.insert(Key::Int(k + 1), v_);
            }
            tb.set(num(p as f64), v.clone());
        } else {
            tb.set(num((len + 1) as f64), v.clone());
        }
        return Ok(vec![v]);
    }
    Ok(vec![Value::Nil])
}

fn api_del(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let (Some(Value::Table(t)), Some(target)) = (args.first(), args.get(1)) {
        let mut tb = t.borrow_mut();
        let len = tb.raw_len();
        let mut found_idx: Option<i64> = None;
        for i in 1..=len {
            let v = tb.map.get(&Key::Int(i)).cloned().unwrap_or(Value::Nil);
            if v.raw_equal(target) {
                found_idx = Some(i);
                break;
            }
        }
        if let Some(i) = found_idx {
            let removed = tb.map.get(&Key::Int(i)).cloned().unwrap_or(Value::Nil);
            for k in i..len {
                let v_ = tb.map.get(&Key::Int(k + 1)).cloned().unwrap_or(Value::Nil);
                if matches!(v_, Value::Nil) {
                    tb.map.remove(&Key::Int(k));
                } else {
                    tb.map.insert(Key::Int(k), v_);
                }
            }
            tb.map.remove(&Key::Int(len));
            return Ok(vec![removed]);
        }
    }
    Ok(vec![Value::Nil])
}

fn api_deli(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let Some(Value::Table(t)) = args.first() {
        let mut tb = t.borrow_mut();
        let len = tb.raw_len();
        let idx = match args.get(1) {
            Some(v) if !matches!(v, Value::Nil) => v.as_number().unwrap_or(0.0) as i64,
            _ => len,
        };
        if idx < 1 || idx > len {
            return Ok(vec![Value::Nil]);
        }
        let removed = tb.map.get(&Key::Int(idx)).cloned().unwrap_or(Value::Nil);
        for k in idx..len {
            let v_ = tb.map.get(&Key::Int(k + 1)).cloned().unwrap_or(Value::Nil);
            if matches!(v_, Value::Nil) {
                tb.map.remove(&Key::Int(k));
            } else {
                tb.map.insert(Key::Int(k), v_);
            }
        }
        tb.map.remove(&Key::Int(len));
        return Ok(vec![removed]);
    }
    Ok(vec![Value::Nil])
}

fn api_count(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = match args.first() {
        Some(Value::Table(t)) => t.clone(),
        _ => return Ok(vec![num(0.0)]),
    };
    if let Some(target) = args.get(1) {
        if !matches!(target, Value::Nil) {
            let tb = t.borrow();
            let len = tb.raw_len();
            let mut c = 0;
            for i in 1..=len {
                if let Some(v) = tb.map.get(&Key::Int(i)) {
                    if v.raw_equal(target) {
                        c += 1;
                    }
                }
            }
            return Ok(vec![num(c as f64)]);
        }
    }
    let n = t.borrow().raw_len() as f64;
    Ok(vec![num(n)])
}

fn api_foreach(i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = match args.first() {
        Some(Value::Table(t)) => t.clone(),
        _ => return Ok(vec![]),
    };
    let f = args.get(1).cloned().unwrap_or(Value::Nil);
    let mut idx: i64 = 1;
    loop {
        let len = t.borrow().raw_len();
        if idx > len {
            break;
        }
        let v = t.borrow().get(&num(idx as f64));
        let _ = i.call_value(&f, vec![v])?;
        let new_len = t.borrow().raw_len();
        if new_len >= len {
            idx += 1;
        }
    }
    Ok(vec![])
}

fn api_all(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    // all() over a STRING iterates its characters as 1-char strings --
    // confirmed via oracle (`for c in all("abc")`); samurise-1.p8.png's
    // embedded parens8 LISP VM tokenizes with `for e in all(" \n\t")`,
    // and an empty iterator here spun its scanner loop forever.
    if let Some(Value::Str(s)) = args.first() {
        let s = Rc::clone(s);
        let pos = std::cell::Cell::new(0usize);
        return Ok(vec![Value::Function(Function::Native(Rc::new(NativeFn {
            name: "all_str",
            func: Box::new(move |_i, _a| {
                let i = pos.get();
                if i >= s.len() {
                    return Ok(vec![nil()]);
                }
                pos.set(i + 1);
                Ok(vec![Value::Str(Rc::from(&s[i..i + 1]))])
            }),
        })))]);
    }
    let t = match args.first() {
        Some(Value::Table(t)) => t.clone(),
        _ => {
            // Return iterator that immediately returns nil
            return Ok(vec![Value::Function(Function::Native(Rc::new(NativeFn {
                name: "all_empty",
                func: Box::new(|_i, _a| Ok(vec![nil()])),
            })))]);
        }
    };
    // Closure-captured state. Generic-for passes (state, control) but we
    // ignore both — the captured state is the source of truth. This keeps
    // us robust to the env-fallback `wrap` shim that re-wraps `all`.
    //
    // del()-during-iteration (a very common PICO-8 pattern) must not skip
    // or double-visit items. Tracking just "did the length shrink" isn't
    // enough to tell WHERE the shrink happened: deleting the current item
    // shifts later items down into this same index (must re-visit it),
    // but deleting a later item leaves this index's content unchanged
    // (must advance normally) -- confirmed against official PICO-8 that
    // the two cases differ. So remember the last-returned VALUE, not just
    // the index, and re-check whether that position still holds it.
    type AllIterState = RefCell<(Rc<Table>, i64, Option<Value>)>;
    let state: Rc<AllIterState> = Rc::new(RefCell::new((Rc::clone(&t), 0i64, None)));
    let iter = Value::Function(Function::Native(Rc::new(NativeFn {
        name: "all_iter",
        func: Box::new(move |_i, _args| {
            let mut st = state.borrow_mut();
            let tref = Rc::clone(&st.0);
            let len = tref.borrow().raw_len();
            let last_idx = st.1;
            let advance = match &st.2 {
                None => true,
                Some(last_val) => {
                    let now_at_last_idx = tref.borrow().get(&num(last_idx as f64));
                    now_at_last_idx.raw_equal(last_val)
                }
            };
            let next_idx = if advance { last_idx + 1 } else { last_idx };
            if next_idx < 1 || next_idx > len {
                return Ok(vec![nil()]);
            }
            let v = tref.borrow().get(&num(next_idx as f64));
            st.1 = next_idx;
            st.2 = Some(v.clone());
            Ok(vec![v])
        }),
    })));
    Ok(vec![iter])
}

fn api_pack(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = Rc::new(RefCell::new(TableInner::new()));
    for (i, v) in args.iter().enumerate() {
        t.borrow_mut().set(num((i + 1) as f64), v.clone());
    }
    t.borrow_mut().set(str_v(b"n"), num(args.len() as f64));
    Ok(vec![Value::Table(t)])
}

fn api_unpack(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = match args.first() {
        Some(Value::Table(t)) => t.clone(),
        _ => return Ok(vec![]),
    };
    let start = opt_int(&args, 1, 1) as i64;
    let end = if args.len() > 2 && !matches!(args[2], Value::Nil) {
        args[2].as_number().unwrap_or(0.0) as i64
    } else {
        let n = t.borrow().get(&str_v(b"n"));
        if let Some(nv) = n.as_number() {
            nv as i64
        } else {
            t.borrow().raw_len()
        }
    };
    let mut out = Vec::new();
    for i in start..=end {
        out.push(t.borrow().get(&num(i as f64)));
    }
    Ok(out)
}

// === Graphics ===

fn api_cls(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = if a.is_empty() || matches!(a[0], Value::Nil) {
        0
    } else {
        (arg_int(&a, 0).unwrap_or(0) & 0xF) as u8
    };
    gfx::cls(i.host(), col);
    Ok(vec![])
}
fn api_pset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = arg_int(&a, 0).unwrap_or(0);
    let y = arg_int(&a, 1).unwrap_or(0);
    let col = get_color(i.host(), &a, 2);
    gfx::put_pixel(&mut i.host().memory, x, y, col);
    Ok(vec![])
}
fn api_pget(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = arg_int(&a, 0).unwrap_or(0);
    let y = arg_int(&a, 1).unwrap_or(0);
    if !(0..128).contains(&x) || !(0..128).contains(&y) {
        return Ok(vec![num(0.0)]);
    }
    let c = i.host().memory.screen_get(x as u8, y as u8);
    Ok(vec![num(c as f64)])
}
fn api_line(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    if a.is_empty() {
        st.line_valid = false;
        return Ok(vec![]);
    }
    if a.len() <= 3 || matches!(a.get(3), Some(Value::Nil)) {
        let x1 = arg_int(&a, 0).unwrap_or(0);
        let y1 = arg_int(&a, 1).unwrap_or(0);
        let col = get_color(st, &a, 2);
        if st.line_valid {
            gfx::draw_line(&mut st.memory, st.line_x, st.line_y, x1, y1, col);
        }
        st.line_x = x1;
        st.line_y = y1;
        st.line_valid = true;
        st.memory.poke16(mem::ADDR_LINE_X, x1 as u16);
        st.memory.poke16(mem::ADDR_LINE_Y, y1 as u16);
        return Ok(vec![]);
    }
    let x0 = arg_int(&a, 0).unwrap_or(0);
    let y0 = arg_int(&a, 1).unwrap_or(0);
    let x1 = arg_int(&a, 2).unwrap_or(0);
    let y1 = arg_int(&a, 3).unwrap_or(0);
    let col = get_color(st, &a, 4);
    gfx::draw_line(&mut st.memory, x0, y0, x1, y1, col);
    st.line_x = x1;
    st.line_y = y1;
    st.line_valid = true;
    st.memory.poke16(mem::ADDR_LINE_X, x1 as u16);
    st.memory.poke16(mem::ADDR_LINE_Y, y1 as u16);
    Ok(vec![])
}
fn api_rect(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 4);
    gfx::rect(
        &mut i.host().memory,
        arg_int(&a, 0).unwrap_or(0),
        arg_int(&a, 1).unwrap_or(0),
        arg_int(&a, 2).unwrap_or(0),
        arg_int(&a, 3).unwrap_or(0),
        col,
    );
    Ok(vec![])
}
fn api_rectfill(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 4);
    gfx::rectfill(
        &mut i.host().memory,
        arg_int(&a, 0).unwrap_or(0),
        arg_int(&a, 1).unwrap_or(0),
        arg_int(&a, 2).unwrap_or(0),
        arg_int(&a, 3).unwrap_or(0),
        col,
    );
    Ok(vec![])
}
fn api_circ(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 3);
    gfx::circ(
        &mut i.host().memory,
        arg_int(&a, 0).unwrap_or(0),
        arg_int(&a, 1).unwrap_or(0),
        opt_int(&a, 2, 4),
        col,
    );
    Ok(vec![])
}
fn api_circfill(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 3);
    gfx::circfill(
        &mut i.host().memory,
        arg_int(&a, 0).unwrap_or(0),
        arg_int(&a, 1).unwrap_or(0),
        opt_int(&a, 2, 4),
        col,
    );
    Ok(vec![])
}
fn api_oval(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 4);
    gfx::oval(
        &mut i.host().memory,
        arg_int(&a, 0).unwrap_or(0),
        arg_int(&a, 1).unwrap_or(0),
        arg_int(&a, 2).unwrap_or(0),
        arg_int(&a, 3).unwrap_or(0),
        col,
    );
    Ok(vec![])
}
fn api_ovalfill(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 4);
    gfx::ovalfill(
        &mut i.host().memory,
        arg_int(&a, 0).unwrap_or(0),
        arg_int(&a, 1).unwrap_or(0),
        arg_int(&a, 2).unwrap_or(0),
        arg_int(&a, 3).unwrap_or(0),
        col,
    );
    Ok(vec![])
}
fn api_spr(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let n = arg_int(&a, 0).unwrap_or(0);
    let x = arg_int(&a, 1).unwrap_or(0);
    let y = arg_int(&a, 2).unwrap_or(0);
    let w = opt_num(&a, 3, 1.0);
    let h = opt_num(&a, 4, 1.0);
    let fx = opt_bool(&a, 5, false);
    let fy = opt_bool(&a, 6, false);
    gfx::spr(&mut i.host().memory, n, x, y, w, h, fx, fy);
    Ok(vec![])
}
fn api_sspr(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let sx = arg_int(&a, 0).unwrap_or(0);
    let sy = arg_int(&a, 1).unwrap_or(0);
    let sw = arg_int(&a, 2).unwrap_or(0);
    let sh = arg_int(&a, 3).unwrap_or(0);
    let dx = arg_int(&a, 4).unwrap_or(0);
    let dy = arg_int(&a, 5).unwrap_or(0);
    let dw = opt_int(&a, 6, sw);
    let dh = opt_int(&a, 7, sh);
    let fx = opt_bool(&a, 8, false);
    let fy = opt_bool(&a, 9, false);
    gfx::sspr(&mut i.host().memory, sx, sy, sw, sh, dx, dy, dw, dh, fx, fy);
    Ok(vec![])
}
fn api_map(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let cx = opt_int(&a, 0, 0);
    let cy = opt_int(&a, 1, 0);
    let sx = opt_int(&a, 2, 0);
    let sy = opt_int(&a, 3, 0);
    let cw = opt_int(&a, 4, 128);
    let ch = opt_int(&a, 5, 64);
    let layer = opt_int(&a, 6, 0);
    gfx::map(&mut i.host().memory, cx, cy, sx, sy, cw, ch, layer);
    Ok(vec![])
}
fn api_mget(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = arg_int(&a, 0).unwrap_or(0);
    let y = arg_int(&a, 1).unwrap_or(0);
    Ok(vec![num(gfx::map_get_wide(&i.host().memory, x, y) as f64)])
}
fn api_mset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = arg_int(&a, 0).unwrap_or(0);
    let y = arg_int(&a, 1).unwrap_or(0);
    let v = (arg_int(&a, 2).unwrap_or(0) & 0xFF) as u8;
    gfx::map_set_wide(&mut i.host().memory, x, y, v);
    Ok(vec![])
}
fn api_sget(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = arg_int(&a, 0).unwrap_or(0);
    let y = arg_int(&a, 1).unwrap_or(0);
    if !(0..128).contains(&x) || !(0..128).contains(&y) {
        return Ok(vec![num(0.0)]);
    }
    Ok(vec![num(
        i.host().memory.sprite_get(x as u8, y as u8) as f64
    )])
}
fn api_sset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = arg_int(&a, 0).unwrap_or(0);
    let y = arg_int(&a, 1).unwrap_or(0);
    let col = get_color(i.host(), &a, 2);
    if (0..128).contains(&x) && (0..128).contains(&y) {
        i.host().memory.sprite_set(x as u8, y as u8, col);
    }
    Ok(vec![])
}
fn api_fget(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let n = arg_int(&a, 0).unwrap_or(0);
    if !(0..256).contains(&n) {
        return Ok(vec![num(0.0)]);
    }
    let flags = i.host().memory.ram[mem::ADDR_FLAGS as usize + n as usize];
    if a.len() <= 1 || matches!(a[1], Value::Nil) {
        return Ok(vec![num(flags as f64)]);
    }
    let bit = arg_int(&a, 1).unwrap_or(0) as u32 & 7;
    Ok(vec![boolv(flags & (1u8 << bit) != 0)])
}
fn api_fset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let n = arg_int(&a, 0).unwrap_or(0);
    if !(0..256).contains(&n) {
        return Ok(vec![]);
    }
    let addr = mem::ADDR_FLAGS as usize + n as usize;
    if a.len() <= 2 || matches!(a.get(2), Some(Value::Nil)) {
        i.host().memory.ram[addr] = (arg_int(&a, 1).unwrap_or(0) & 0xFF) as u8;
    } else {
        let bit = arg_int(&a, 1).unwrap_or(0) as u32 & 7;
        if a[2].truthy() {
            i.host().memory.ram[addr] |= 1u8 << bit;
        } else {
            i.host().memory.ram[addr] &= !(1u8 << bit);
        }
    }
    Ok(vec![])
}
fn api_print(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let txt = a
        .first()
        .cloned()
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| Rc::from(b"".as_slice()));
    let st = i.host();
    let (x, y, col) = if a.len() <= 1 || matches!(a.get(1), Some(Value::Nil)) {
        let cx = st.memory.ram[mem::ADDR_CURSOR_X as usize] as i32;
        let cy = st.memory.ram[mem::ADDR_CURSOR_Y as usize] as i32;
        let col = st.memory.ram[mem::ADDR_COLOR as usize] & 0x0F;
        (cx, cy, col)
    } else if a.len() <= 2 || matches!(a.get(2), Some(Value::Nil)) {
        let cx = st.memory.ram[mem::ADDR_CURSOR_X as usize] as i32;
        let cy = st.memory.ram[mem::ADDR_CURSOR_Y as usize] as i32;
        let col = get_color(st, &a, 1);
        (cx, cy, col)
    } else {
        let x = arg_int(&a, 1).unwrap_or(0);
        let y = arg_int(&a, 2).unwrap_or(0);
        let col = get_color(st, &a, 3);
        (x, y, col)
    };
    // P8SCII `\^1`..`\^9` are frame-pause control codes -- golfed carts
    // replace their whole flip() with `?"\^1\^c"` (pause a frame + clear;
    // lv-2.p8.png). Count each as a frame tick so a headless host's flip
    // budget (see api_flip) also ends explicit print-pause main loops.
    let mut k = 0;
    while k + 1 < txt.len() {
        if txt[k] == 0x06 && txt[k + 1].is_ascii_digit() && txt[k + 1] != b'0' {
            st.flip_count += 1;
            st.frame_count += 1;
            if st.flip_limit != 0 && st.flip_count > st.flip_limit {
                return Err(RtError::msg(FLIP_LIMIT_MARKER));
            }
        }
        k += 1;
    }
    let r = gfx::draw_text(&mut st.memory, &txt, x, y, col);
    // Confirmed against official PICO-8: print() always persists the
    // cursor registers afterward, regardless of which argument form was
    // used -- cursor_x resets to this call's starting x (not wherever the
    // text ended up), and cursor_y advances by 6 PER LINE printed (not a
    // flat +6), both confirmed via a "hello\nb" probe (cx stayed at
    // start_x=20, cy became start_y+12 for its 2 lines, not +6).
    let num_lines = 1 + txt.iter().filter(|&&b| b == b'\n').count() as i32;
    st.memory.ram[mem::ADDR_CURSOR_X as usize] = (x & 0xFF) as u8;
    st.memory.ram[mem::ADDR_CURSOR_Y as usize] = ((y + 6 * num_lines) & 0xFF) as u8;
    Ok(vec![num(r as f64)])
}
fn api_cursor(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = opt_int(&a, 0, 0);
    let y = opt_int(&a, 1, 0);
    let st = i.host();
    st.memory.ram[mem::ADDR_CURSOR_X as usize] = (x & 0xFF) as u8;
    st.memory.ram[mem::ADDR_CURSOR_Y as usize] = (y & 0xFF) as u8;
    if a.len() > 2 && !matches!(a[2], Value::Nil) {
        st.memory.ram[mem::ADDR_COLOR as usize] = (arg_int(&a, 2).unwrap_or(0) & 0xF) as u8;
    }
    Ok(vec![])
}
fn api_color(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    let prev = st.memory.ram[mem::ADDR_COLOR as usize];
    let col = opt_int(&a, 0, 6);
    st.memory.ram[mem::ADDR_COLOR as usize] = (col & 0xFF) as u8;
    Ok(vec![num(prev as f64)])
}
fn api_camera(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    let prev_x = st.memory.peek16(mem::ADDR_CAMERA_X) as i16 as i32;
    let prev_y = st.memory.peek16(mem::ADDR_CAMERA_Y) as i16 as i32;
    let x = opt_int(&a, 0, 0) as i16 as u16;
    let y = opt_int(&a, 1, 0) as i16 as u16;
    st.memory.poke16(mem::ADDR_CAMERA_X, x);
    st.memory.poke16(mem::ADDR_CAMERA_Y, y);
    Ok(vec![num(prev_x as f64), num(prev_y as f64)])
}
fn api_clip(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    let prev_x = st.memory.ram[mem::ADDR_CLIP_LEFT as usize] as i32;
    let prev_y = st.memory.ram[mem::ADDR_CLIP_TOP as usize] as i32;
    let prev_w = st.memory.ram[mem::ADDR_CLIP_RIGHT as usize] as i32 - prev_x;
    let prev_h = st.memory.ram[mem::ADDR_CLIP_BOTTOM as usize] as i32 - prev_y;
    if a.is_empty() || matches!(a[0], Value::Nil) {
        st.memory.ram[mem::ADDR_CLIP_LEFT as usize] = 0;
        st.memory.ram[mem::ADDR_CLIP_TOP as usize] = 0;
        st.memory.ram[mem::ADDR_CLIP_RIGHT as usize] = 128;
        st.memory.ram[mem::ADDR_CLIP_BOTTOM as usize] = 128;
    } else {
        let mut x = arg_int(&a, 0).unwrap_or(0).max(0);
        let mut y = arg_int(&a, 1).unwrap_or(0).max(0);
        let mut x1 = (x + arg_int(&a, 2).unwrap_or(0)).min(128);
        let mut y1 = (y + arg_int(&a, 3).unwrap_or(0)).min(128);
        let clip_prev = opt_bool(&a, 4, false);
        if clip_prev {
            x = x.max(st.memory.ram[mem::ADDR_CLIP_LEFT as usize] as i32);
            y = y.max(st.memory.ram[mem::ADDR_CLIP_TOP as usize] as i32);
            x1 = x1.min(st.memory.ram[mem::ADDR_CLIP_RIGHT as usize] as i32);
            y1 = y1.min(st.memory.ram[mem::ADDR_CLIP_BOTTOM as usize] as i32);
        }
        st.memory.ram[mem::ADDR_CLIP_LEFT as usize] = x as u8;
        st.memory.ram[mem::ADDR_CLIP_TOP as usize] = y as u8;
        st.memory.ram[mem::ADDR_CLIP_RIGHT as usize] = x1 as u8;
        st.memory.ram[mem::ADDR_CLIP_BOTTOM as usize] = y1 as u8;
    }
    Ok(vec![
        num(prev_x as f64),
        num(prev_y as f64),
        num(prev_w as f64),
        num(prev_h as f64),
    ])
}
fn api_pal(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    if a.is_empty() || matches!(a[0], Value::Nil) {
        for k in 0..16 {
            st.memory.ram[mem::ADDR_DRAW_PAL as usize + k] = k as u8;
            st.memory.ram[mem::ADDR_SCREEN_PAL as usize + k] = k as u8;
        }
        st.memory.ram[mem::ADDR_DRAW_PAL as usize] |= 0x10;
        st.memory.poke16(mem::ADDR_FILL_PAT, 0);
        st.memory.ram[mem::ADDR_FILL_PAT as usize + 2] = 0;
        return Ok(vec![]);
    }
    if let Some(Value::Table(t)) = a.first() {
        let p = opt_int(&a, 1, 0);
        if p == 2 {
            // Secondary/pen palette (used by fillp's alternate color) --
            // confirmed against official PICO-8 that this must NOT touch
            // the draw palette at 0x5f00. Its own storage/effect isn't
            // implemented yet; a safe no-op is still more correct than
            // corrupting the draw palette.
            return Ok(vec![]);
        }
        // Confirmed against official PICO-8: out-of-range table keys wrap
        // via `% 16` (pal({[20]=5}) remaps color 4, not dropped).
        for (key, v) in t.borrow().map.iter() {
            let Key::Int(key) = key else { continue };
            let k = key.rem_euclid(16) as usize;
            if !matches!(v, Value::Nil) {
                let val = (v.as_number().unwrap_or(0.0) as i32 & 0xF) as u8;
                if p == 1 {
                    st.memory.ram[mem::ADDR_SCREEN_PAL as usize + k] = val;
                } else {
                    let trans = st.memory.ram[mem::ADDR_DRAW_PAL as usize + k] & 0x10;
                    st.memory.ram[mem::ADDR_DRAW_PAL as usize + k] = val | trans;
                }
            }
        }
        return Ok(vec![]);
    }
    let c0 = arg_int(&a, 0).unwrap_or(0) as u8 & 0xF;
    let c1 = arg_int(&a, 1).unwrap_or(0) as u8 & 0xF;
    let p = opt_int(&a, 2, 0);
    if p == 2 {
        // See the table-form comment above: p=2 must not touch 0x5f00.
        return Ok(vec![]);
    }
    if p == 1 {
        st.memory.ram[mem::ADDR_SCREEN_PAL as usize + c0 as usize] = c1;
    } else {
        let trans = st.memory.ram[mem::ADDR_DRAW_PAL as usize + c0 as usize] & 0x10;
        st.memory.ram[mem::ADDR_DRAW_PAL as usize + c0 as usize] = c1 | trans;
    }
    Ok(vec![])
}
fn api_palt(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    if a.is_empty() || matches!(a[0], Value::Nil) {
        for k in 0..16 {
            st.memory.ram[mem::ADDR_DRAW_PAL as usize + k] &= 0x0F;
        }
        st.memory.ram[mem::ADDR_DRAW_PAL as usize] |= 0x10;
        return Ok(vec![]);
    }
    if a.len() <= 1 || matches!(a.get(1), Some(Value::Nil)) {
        let bits = arg_int(&a, 0).unwrap_or(0) as u32;
        for k in 0..16 {
            // Confirmed against official PICO-8: bit (15-k) controls color
            // k (bit 0 = color 15 ... bit 15 = color 0), not bit k directly.
            if bits & (1 << (15 - k)) != 0 {
                st.memory.ram[mem::ADDR_DRAW_PAL as usize + k] |= 0x10;
            } else {
                st.memory.ram[mem::ADDR_DRAW_PAL as usize + k] &= 0x0F;
            }
        }
        return Ok(vec![]);
    }
    let col = arg_int(&a, 0).unwrap_or(0) as usize & 0xF;
    if a[1].truthy() {
        st.memory.ram[mem::ADDR_DRAW_PAL as usize + col] |= 0x10;
    } else {
        st.memory.ram[mem::ADDR_DRAW_PAL as usize + col] &= 0x0F;
    }
    Ok(vec![])
}
fn api_fillp(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    let p = opt_num(&a, 0, 0.0);
    let int_val = safe_to_i32(p) as u32;
    st.memory
        .poke16(mem::ADDR_FILL_PAT, (int_val & 0xFFFF) as u16);
    let trans = if int_val & 0x10000 != 0 { 1 } else { 0 };
    st.memory.ram[mem::ADDR_FILL_PAT as usize + 2] = trans;
    Ok(vec![])
}

// === Input ===

fn api_btn(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    note_btn_poll(st)?;
    if a.is_empty() || matches!(a[0], Value::Nil) {
        let p0 = (st.input.btn_state[0] & 0x3F) as u32;
        let p1 = ((st.input.btn_state[1] & 0x3F) as u32) << 8;
        return Ok(vec![num((p0 | p1) as f64)]);
    }
    let b = (arg_int(&a, 0).unwrap_or(0) as u8) & 7;
    // We only track 2 players; PICO-8 supports up to 8 — for any unsupported
    // player slot return false (NOT alias to p & 1, which would make higher
    // player numbers spuriously read player 0/1 state).
    let p_raw = opt_int(&a, 1, 0) as u32;
    if p_raw > 1 {
        return Ok(vec![boolv(false)]);
    }
    Ok(vec![boolv(st.input.btn(b, p_raw as u8))])
}
/// Input-wait busy loops (`while(not btn(4))do print(...)end`,
/// onek_landscape06-0.p8.png) poll btn()/btnp() without ever flipping --
/// official PICO-8 keeps servicing frames during such loops, so they
/// terminate on real hardware. Headlessly, count sustained polling as
/// synthetic frame ticks so the flip budget (see api_flip) ends the loop
/// as "ran N frames" instead of hanging forever. Inactive (flip_limit==0)
/// on interactive/WASM hosts.
fn note_btn_poll(st: &mut crate::state::PicoState) -> Result<(), RtError> {
    if st.flip_limit == 0 {
        return Ok(());
    }
    st.btn_poll_count += 1;
    if st.btn_poll_count >= 64 {
        st.btn_poll_count = 0;
        st.flip_count += 1;
        st.frame_count += 1;
        if st.flip_count > st.flip_limit {
            return Err(RtError::msg(FLIP_LIMIT_MARKER));
        }
    }
    Ok(())
}

fn api_btnp(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    note_btn_poll(st)?;
    if a.is_empty() || matches!(a[0], Value::Nil) {
        let mut p0: u32 = 0;
        let mut p1: u32 = 0;
        for b in 0..6u8 {
            if st.input.btnp(b, 0, &st.memory) {
                p0 |= 1 << b;
            }
            if st.input.btnp(b, 1, &st.memory) {
                p1 |= 1 << b;
            }
        }
        return Ok(vec![num((p0 | (p1 << 8)) as f64)]);
    }
    let b = (arg_int(&a, 0).unwrap_or(0) as u8) & 7;
    let p_raw = opt_int(&a, 1, 0) as u32;
    if p_raw > 1 {
        return Ok(vec![boolv(false)]);
    }
    Ok(vec![boolv(st.input.btnp(b, p_raw as u8, &st.memory))])
}

// === Memory ===

fn api_peek(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_addr(&a, 0);
    let n = opt_int(&a, 1, 1).max(1) as usize;
    if n == 1 {
        return Ok(vec![num(i.host().memory.peek(addr) as f64)]);
    }
    let count = n.min(8192);
    let mut out = Vec::with_capacity(count);
    for k in 0..count {
        out.push(num(i.host().memory.peek(addr.wrapping_add(k as u16)) as f64));
    }
    Ok(out)
}
fn api_poke(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_addr(&a, 0);
    for (k, v) in a.iter().enumerate().skip(1) {
        let val = (v.as_number().unwrap_or(0.0) as i32 & 0xFF) as u8;
        i.host().memory.poke(addr.wrapping_add((k - 1) as u16), val);
    }
    Ok(vec![])
}
fn api_peek2(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_addr(&a, 0);
    Ok(vec![num(i.host().memory.peek16(addr) as f64)])
}
fn api_poke2(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_addr(&a, 0);
    let val = (safe_to_i32(arg_num(&a, 1).unwrap_or(0.0)) as u32 & 0xFFFF) as u16;
    i.host().memory.poke16(addr, val);
    Ok(vec![])
}
fn api_peek4(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_addr(&a, 0);
    let raw = i.host().memory.peek32(addr);
    let fixed = raw as i32;
    Ok(vec![num(fixed as f64 / 65536.0)])
}
fn api_poke4(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_addr(&a, 0);
    let v = arg_num(&a, 1).unwrap_or(0.0);
    let scaled = v * 65536.0;
    let fixed = if scaled >= i32::MAX as f64 {
        i32::MAX
    } else if scaled <= i32::MIN as f64 {
        i32::MIN
    } else {
        scaled as i32
    };
    i.host().memory.poke32(addr, fixed as u32);
    Ok(vec![])
}
fn api_memcpy(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    i.host().memory.memcpy(
        arg_addr(&a, 0),
        arg_addr(&a, 1),
        arg_int(&a, 2).unwrap_or(0) as u16,
    );
    Ok(vec![])
}
fn api_memset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    i.host().memory.memset(
        arg_addr(&a, 0),
        (arg_int(&a, 1).unwrap_or(0) & 0xFF) as u8,
        arg_int(&a, 2).unwrap_or(0) as u16,
    );
    Ok(vec![])
}
fn api_reload(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let dst = if a.is_empty() || matches!(a[0], Value::Nil) {
        0
    } else {
        arg_int(&a, 0).unwrap_or(0) as u16
    };
    let src = if a.len() < 2 || matches!(a[1], Value::Nil) {
        0
    } else {
        arg_int(&a, 1).unwrap_or(0) as u16
    };
    let len = if a.len() < 3 || matches!(a[2], Value::Nil) {
        0x4300
    } else {
        arg_int(&a, 2).unwrap_or(0) as u16
    };
    i.host().memory.reload(dst, src, len);
    Ok(vec![])
}
fn api_cstore(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let dst = if a.is_empty() || matches!(a[0], Value::Nil) {
        0
    } else {
        arg_int(&a, 0).unwrap_or(0) as u16
    };
    let src = if a.len() < 2 || matches!(a[1], Value::Nil) {
        0
    } else {
        arg_int(&a, 1).unwrap_or(0) as u16
    };
    let len = if a.len() < 3 || matches!(a[2], Value::Nil) {
        0x4300
    } else {
        arg_int(&a, 2).unwrap_or(0) as u16
    };
    let st = i.host();
    for k in 0..len {
        let dst_addr = dst.wrapping_add(k);
        // Confirmed against official PICO-8: cstore silently skips writes
        // at/beyond 0x4300 (the compiled-code region) instead of erroring
        // or writing through -- verified with a destination range that
        // straddles the boundary, where only the below-0x4300 bytes land.
        if dst_addr >= mem::ADDR_GENERAL {
            continue;
        }
        st.memory.rom[dst_addr as usize] = st.memory.ram[src.wrapping_add(k) as usize];
    }
    Ok(vec![])
}

// === Audio ===

fn api_sfx(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let id = opt_int(&a, 0, -1);
    let ch = opt_int(&a, 1, -1);
    let off = opt_int(&a, 2, 0);
    let st = i.host();
    let mem = &st.memory;
    st.audio.play_sfx(mem, id, ch, off);
    Ok(vec![])
}
fn api_music(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let pat = opt_int(&a, 0, -1);
    let fade = opt_int(&a, 1, 0);
    // -1 here means "omitted" (see play_music) -- distinct from an
    // explicit 0, which means "reserve no channels".
    let mask = opt_int(&a, 2, -1);
    let st = i.host();
    let mem = &st.memory;
    st.audio.play_music(mem, pat, fade, mask);
    Ok(vec![])
}

// === System ===

fn api_stat(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let n = opt_int(&a, 0, 0);
    let st = i.host();
    let v = match n {
        0 => num(64.0),
        1 => num(0.5),
        4 => str_v(b""),
        5 => str_v(b"0.2.3"),
        6 => str_v(b""),
        7 => num(st.target_fps as f64),
        // Confirmed against official PICO-8 (sfx(3,0); stat(16)==3): stat
        // 16-19 is the per-channel sfx index, 20-23 is the note index --
        // the reverse of what these two branches previously returned.
        16..=19 => {
            let c = &st.audio.channels[(n - 16) as usize];
            if !c.finished && c.sfx_id >= 0 {
                num(c.sfx_id as f64)
            } else {
                num(-1.0)
            }
        }
        20..=23 => {
            let c = &st.audio.channels[(n - 20) as usize];
            if !c.finished && c.sfx_id >= 0 {
                num(c.note_index as f64)
            } else {
                num(0.0)
            }
        }
        24 => num(st.audio.music_state.pattern as f64),
        26 => num(st.audio.music_state.tick as f64),
        30 => boolv(st.input.key_chars_len > 0),
        31 => str_v(b""),
        32 => num(if st.memory.ram[0x5F2D] & 1 != 0 {
            st.input.mouse_x as f64
        } else {
            0.0
        }),
        33 => num(if st.memory.ram[0x5F2D] & 1 != 0 {
            st.input.mouse_y as f64
        } else {
            0.0
        }),
        34 => num(if st.memory.ram[0x5F2D] & 1 != 0 {
            st.input.mouse_buttons as f64
        } else {
            0.0
        }),
        36 => num(if st.memory.ram[0x5F2D] & 1 != 0 {
            st.input.mouse_wheel as f64
        } else {
            0.0
        }),
        46..=49 => {
            let c = &st.audio.channels[(n - 46) as usize];
            if !c.finished && c.sfx_id >= 0 {
                num(c.note_index as f64)
            } else {
                num(0.0)
            }
        }
        50..=53 => {
            let c = &st.audio.channels[(n - 50) as usize];
            if !c.finished && c.sfx_id >= 0 {
                num(c.sfx_id as f64)
            } else {
                num(-1.0)
            }
        }
        54 => num(st.audio.music_state.pattern as f64),
        55 => num(st.audio.music_state.total_patterns as f64),
        56 => num(st.audio.music_state.tick as f64),
        57 => boolv(st.audio.music_state.playing),
        // stat(80..85) UTC / stat(90..95) local wall-clock date-time
        // components (year, month, day, hour, minute, second). Carts show
        // these on clocks/calendars (kokoroko-3.p8.png); real values can't
        // be reproduced deterministically in tests, so return a fixed,
        // plausible timestamp rather than nil (which crashed carts that
        // concatenate the components).
        // (Devkit stat(30)/stat(31) are handled above with the input
        // state; both return their oracle-confirmed boolean/string types.)
        // stat(28,code): raw keyboard scancode held? stat(120)/stat(121):
        // stdin/serial data pending? All false headlessly, and all
        // BOOLEANS on official (oracle-confirmed) -- the generic numeric
        // 0 fallback is truthy in Lua and spun `while stat(120) do`-style
        // drain loops forever.
        28 => boolv(false),
        120 | 121 => boolv(false),
        108 => num(st.serial_queued as f64),
        80 | 90 => num(2024.0),
        81 | 91 => num(1.0),
        82 | 92 => num(1.0),
        83 | 93 => num(12.0),
        84 | 94 => num(0.0),
        85 | 95 => num(0.0),
        // Confirmed against official PICO-8: stat(100) (breadcrumb label)
        // is nil when unset, not the generic 0 fallback other unhandled
        // codes use.
        100 => nil(),
        _ => num(0.0),
    };
    Ok(vec![v])
}
fn api_time(i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(i.host().elapsed_time)])
}
fn api_printh(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let v = a.first().cloned().unwrap_or(Value::Nil);
        println!("{}", display_string(&v));
    }
    Ok(vec![])
}
fn api_cartdata(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![boolv(true)])
}
fn api_dget(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let idx = arg_int(&a, 0).unwrap_or(0);
    if !(0..64).contains(&idx) {
        return Ok(vec![num(0.0)]);
    }
    let addr = mem::ADDR_CART_DATA + (idx as u16) * 4;
    let raw = i.host().memory.peek32(addr);
    let fixed = raw as i32;
    Ok(vec![num(fixed as f64 / 65536.0)])
}
fn api_dset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let idx = arg_int(&a, 0).unwrap_or(0);
    let v = arg_num(&a, 1).unwrap_or(0.0);
    if !(0..64).contains(&idx) {
        return Ok(vec![]);
    }
    let addr = mem::ADDR_CART_DATA + (idx as u16) * 4;
    let scaled = v * 65536.0;
    let fixed = if scaled >= i32::MAX as f64 {
        i32::MAX
    } else if scaled <= i32::MIN as f64 {
        i32::MIN
    } else {
        scaled as i32
    };
    i.host().memory.poke32(addr, fixed as u32);
    Ok(vec![])
}
// === Coroutines (see coroutine.rs for the threading model) ===
// The Lua-visible coroutine value is an empty table whose Rc identity
// keys a registry in the interpreter. (Official reports type "thread";
// this shim reports "table" -- carts observably relying on that
// distinction haven't been seen in the corpus.)

fn co_lookup(
    i: &Interp,
    v: Option<&Value>,
) -> Option<Rc<crate::pico_lua::coroutine::CoroutineHandle>> {
    if let Some(Value::Table(t)) = v {
        return i.coroutines.get(&(Rc::as_ptr(t) as usize)).cloned();
    }
    None
}

fn api_cocreate(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let f = a.first().cloned().unwrap_or(Value::Nil);
    if !matches!(f, Value::Function(_)) {
        return Err(RtError::msg("cocreate: argument must be a function"));
    }
    let handle = crate::pico_lua::coroutine::cocreate(i, f).map_err(RtError::msg)?;
    let t = Rc::new(RefCell::new(TableInner::new()));
    i.coroutines.insert(Rc::as_ptr(&t) as usize, handle);
    Ok(vec![Value::Table(t)])
}

fn api_coresume(i: &mut Interp, mut a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let Some(co) = co_lookup(i, a.first()) else {
        return Ok(vec![boolv(false), str_v(b"cannot resume non-coroutine")]);
    };
    let args: Vec<Value> = if a.is_empty() {
        Vec::new()
    } else {
        a.drain(1..).collect()
    };
    match crate::pico_lua::coroutine::coresume(i, &co, args) {
        Ok(vals) => {
            let mut out = vec![boolv(true)];
            out.extend(vals);
            Ok(out)
        }
        Err(msg) => Ok(vec![boolv(false), str_v(msg.as_bytes())]),
    }
}

fn api_costatus(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    use crate::pico_lua::coroutine::CoStatus;
    let s = match co_lookup(i, a.first()).map(|c| c.status.get()) {
        Some(CoStatus::Suspended) => b"suspended".as_slice(),
        Some(CoStatus::Running) => b"running".as_slice(),
        Some(CoStatus::Dead) | None => b"dead".as_slice(),
    };
    Ok(vec![str_v(s)])
}

fn api_yield(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    crate::pico_lua::coroutine::coyield(i, a).map_err(RtError::msg)
}

fn api_menuitem(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![])
}
fn api_extcmd(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![])
}
// `load(filename,[breadcrumb],[param_str])` loads and starts running a
// *different* cart -- a multi-cart "warp" mechanism. Confirmed real via
// oracle (calling it with a nonexistent cart name doesn't error; execution
// just continues past it). This engine only ever loads one cart per
// invocation and has no concept of switching carts at runtime, so this is
// stubbed as a no-op -- confirmed real-world impact unblocking a crash on
// a real corpus cart (solitomb-2.p8.png: `load'solitomb_title'` in _init).
fn api_load(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![])
}
// `tline(x0,y0,x1,y1,mx,my,[mdx,mdy],[layers])` draws a textured line,
// sampling colors from the map/sprite region as it steps -- confirmed a
// real API function via oracle (doesn't error), but stubbed as a no-op:
// getting its raster algorithm (exact per-pixel step count, coordinate
// space across PICO-8 versions, the optional `layers` mask) right needs
// careful oracle verification against real sprite/map data this session's
// probe carts don't have. Unblocks a real corpus cart from crashing
// (tomorrow-6.p8.png) at the cost of not drawing anything -- same
// no-crash-first tradeoff as the other host-integration stubs here.
fn api_tline(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![])
}
// `_set_fps(n)` overrides the target frame rate to an arbitrary value
// rather than the usual 30/60 toggle -- confirmed real via oracle (doesn't
// error), but its effect on `stat(7)` couldn't be pinned down (a probe
// calling `_set_fps(45)` still reported `stat(7)` as 30 every frame, so
// whatever it changes isn't reflected there). Stubbed as a no-op like the
// other host-integration functions above: this engine's frame loop doesn't
// do real-time pacing at all (run-cart/lib.rs just step a fixed number of
// frames), so the only observable requirement is "don't crash calling it".
fn api_set_fps(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![])
}
// `serial(addr, len, channel)` -- host GPIO/serial data export, confirmed
// a real (if obscure) API function via oracle. Stubbed as a no-op: it has
// no meaningful effect without an actual host-side serial/file channel to
// write to, which this engine doesn't implement.
// `readrom(addr, len)` -- returns `len` bytes of cart ROM as a string.
// NOTE: not oracle-lockable -- the 0.2.7 official binary reports readrom
// as nil under `-x` (headless export) even though it works in normal
// mode and is documented; implemented to the documented semantics
// because samurise-1.p8.png's embedded LISP VM boots its stdlib with
// `(parens8 (readrom 0x2000 0x0fa3))`.
fn api_readrom(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_addr(&a, 0) as usize;
    let len = opt_int(&a, 1, 0).max(0) as usize;
    let rom = &i.host().memory.rom;
    let end = (addr + len).min(rom.len());
    if addr >= end {
        return Ok(vec![str_v(b"")]);
    }
    Ok(vec![str_v(&rom[addr..end])])
}

fn api_serial(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    // Queue accounting only: stat(108) reports cumulative bytes queued
    // (confirmed via oracle -- each serial(ch,addr,len) call adds len).
    let len = opt_int(&a, 2, 0).max(0) as u32;
    i.host().serial_queued = i.host().serial_queued.saturating_add(len);
    Ok(vec![])
}
/// Marker message for a flip() call past the host's frame budget --
/// headless hosts (run-cart) treat an error carrying this message as
/// "explicit-flip main loop ran its N frames clean", not a cart crash.
/// Old-style carts run their entire game as `::_:: ... flip() goto _`
/// at top level and would otherwise hang a headless run forever.
pub const FLIP_LIMIT_MARKER: &str = "__picor_flip_limit__";

fn api_flip(i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    st.flip_count += 1;
    st.frame_count += 1;
    st.elapsed_time += 1.0 / st.target_fps as f64;
    if st.flip_limit != 0 && st.flip_count > st.flip_limit {
        return Err(RtError::msg(FLIP_LIMIT_MARKER));
    }
    Ok(vec![])
}
fn api_reset(i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    i.host().memory.init_draw_state();
    Ok(vec![])
}
fn api_stop(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![])
}
