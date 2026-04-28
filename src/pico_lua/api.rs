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

fn rc_str(s: &str) -> Rc<[u8]> { Rc::from(s.as_bytes()) }
fn nil() -> Value { Value::Nil }
fn num(n: f64) -> Value { Value::Number(n) }
fn boolv(b: bool) -> Value { Value::Bool(b) }
fn str_v(b: &[u8]) -> Value { Value::Str(Rc::from(b)) }

fn arg_num(args: &[Value], i: usize) -> Option<f64> {
    args.get(i).and_then(|v| v.as_number())
}
fn arg_int(args: &[Value], i: usize) -> Option<i32> {
    arg_num(args, i).map(|f| safe_to_i32(f))
}
fn opt_int(args: &[Value], i: usize, d: i32) -> i32 {
    if i >= args.len() || matches!(args[i], Value::Nil) { d } else { arg_int(args, i).unwrap_or(d) }
}
fn opt_num(args: &[Value], i: usize, d: f64) -> f64 {
    if i >= args.len() || matches!(args[i], Value::Nil) { d } else { arg_num(args, i).unwrap_or(d) }
}
fn opt_bool(args: &[Value], i: usize, d: bool) -> bool {
    if i >= args.len() || matches!(args[i], Value::Nil) { d } else { args[i].truthy() }
}
fn arg_str(args: &[Value], i: usize) -> Option<Rc<[u8]>> {
    args.get(i).and_then(|v| v.as_str())
}

fn safe_to_i32(f: f64) -> i32 {
    if f.is_nan() { return 0; }
    if f >= i32::MAX as f64 { return i32::MAX; }
    if f <= i32::MIN as f64 { return i32::MIN; }
    f as i32
}

fn to_fixed(v: f64) -> i32 {
    let scaled = v * 65536.0;
    if scaled.is_nan() { return 0; }
    if scaled > i32::MAX as f64 || scaled < i32::MIN as f64 {
        let wide = scaled as i64;
        return wide as i32;
    }
    scaled as i32
}
fn from_fixed(v: i32) -> f64 { v as f64 / 65536.0 }

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
    let mut set = |name: &'static str, f: fn(&mut Interp, Vec<Value>) -> Result<Vec<Value>, RtError>| {
        let nf = Rc::new(NativeFn {
            name,
            func: Box::new(f),
        });
        g.set(Value::Str(rc_str(name)), Value::Function(Function::Native(nf)));
    };

    // === stdlib ===
    set("type", lua_type);
    set("tostring", lua_tostring);
    set("tonumber", lua_tonumber);
    set("pairs", lua_pairs);
    set("ipairs", lua_ipairs);
    set("next", lua_next);
    set("select", lua_select);
    set("setmetatable", lua_setmetatable);
    set("getmetatable", lua_getmetatable);
    set("rawget", lua_rawget);
    set("rawset", lua_rawset);
    set("rawequal", lua_rawequal);
    set("rawlen", lua_rawlen);
    set("error", lua_error);
    set("pcall", lua_pcall);
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
    set("menuitem", api_menuitem);
    set("extcmd", api_extcmd);
    set("flip", api_flip);
    set("reset", api_reset);
    set("stop", api_stop);

    drop(g);

    // Install a `string` library table for s:sub() etc. style usage
    let strtab = Rc::new(RefCell::new(TableInner::new()));
    {
        let mut s = strtab.borrow_mut();
        let make = |name: &'static str, f: fn(&mut Interp, Vec<Value>) -> Result<Vec<Value>, RtError>| {
            Value::Function(Function::Native(Rc::new(NativeFn { name, func: Box::new(f) })))
        };
        s.set(str_v(b"sub"), make("sub", api_sub));
        s.set(str_v(b"len"), make("len", str_len));
        s.set(str_v(b"lower"), make("lower", str_lower));
        s.set(str_v(b"upper"), make("upper", str_upper));
        s.set(str_v(b"rep"), make("rep", str_rep));
        s.set(str_v(b"format"), make("format", str_format));
    }
    globals.borrow_mut().set(str_v(b"string"), Value::Table(strtab));
}

// === stdlib bodies ===

fn lua_type(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if args.is_empty() { return Ok(vec![]); }
    Ok(vec![str_v(args[0].type_name().as_bytes())])
}

fn lua_tostring(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = args.into_iter().next().unwrap_or(Value::Nil);
    let s = match &v {
        Value::Nil => "nil".to_string(),
        Value::Bool(b) => if *b { "true".into() } else { "false".into() },
        Value::Number(n) => number_to_str(*n),
        Value::Str(b) => return Ok(vec![Value::Str(Rc::clone(b))]),
        Value::Table(t) => format!("table: 0x{:x}", Rc::as_ptr(t) as usize),
        Value::Function(f) => format!("function: 0x{:x}", f.identity()),
    };
    Ok(vec![str_v(s.as_bytes())])
}

fn lua_tonumber(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = args.first().cloned().unwrap_or(Value::Nil);
    Ok(vec![match v.as_number() { Some(n) => num(n), None => nil() }])
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
            let t = args.get(0).cloned().unwrap_or(Value::Nil);
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
    let t = match args.get(0) {
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
            if k == pk { found = true; }
        }
    }
    Ok(vec![nil()])
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
    if args.is_empty() { return Err(RtError::msg("select: missing args")); }
    let first = args.remove(0);
    if let Value::Str(s) = &first {
        if &**s == b"#" {
            return Ok(vec![num(args.len() as f64)]);
        }
    }
    let i = first.as_number().ok_or_else(|| RtError::msg("select: bad index"))? as i64;
    let n = args.len() as i64;
    let start = if i < 0 { (n + i).max(0) } else { (i - 1).max(0) };
    Ok(args.into_iter().skip(start as usize).collect())
}

fn lua_setmetatable(_i: &mut Interp, mut args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if args.len() < 2 { return Err(RtError::msg("setmetatable: 2 args")); }
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
    if let Some(Value::Table(t)) = args.get(0) {
        if let Some(mt) = &t.borrow().metatable {
            return Ok(vec![Value::Table(Rc::clone(mt))]);
        }
    }
    Ok(vec![nil()])
}

fn lua_rawget(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let (Some(Value::Table(t)), Some(k)) = (args.get(0), args.get(1)) {
        return Ok(vec![t.borrow().get(k)]);
    }
    Ok(vec![nil()])
}

fn lua_rawset(_i: &mut Interp, mut args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if args.len() < 3 { return Err(RtError::msg("rawset: 3 args")); }
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
    let a = args.get(0).cloned().unwrap_or(Value::Nil);
    let b = args.get(1).cloned().unwrap_or(Value::Nil);
    Ok(vec![boolv(a.raw_equal(&b))])
}

fn lua_rawlen(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    match args.get(0) {
        Some(Value::Str(s)) => Ok(vec![num(s.len() as f64)]),
        Some(Value::Table(t)) => Ok(vec![num(t.borrow().raw_len() as f64)]),
        _ => Err(RtError::msg("rawlen: needs table or string")),
    }
}

fn lua_error(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = args.into_iter().next().unwrap_or(Value::Nil);
    Err(RtError { value: v })
}

fn lua_pcall(i: &mut Interp, mut args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if args.is_empty() { return Err(RtError::msg("pcall: missing function")); }
    let f = args.remove(0);
    match i.call_value(&f, args) {
        Ok(mut vs) => {
            let mut out = vec![boolv(true)];
            out.append(&mut vs);
            Ok(out)
        }
        Err(e) => Ok(vec![boolv(false), e.value]),
    }
}

fn lua_assert(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = args.get(0).cloned().unwrap_or(Value::Nil);
    if !v.truthy() {
        let msg = args.get(1).cloned().unwrap_or(Value::Str(rc_str("assertion failed!")));
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
    if dx == 0.0 && dy == 0.0 { return Ok(vec![num(0.25)]); }
    // PICO-8 atan2 returns turns in [0,1) with y inverted
    let r = (-dy).atan2(dx);
    let mut t = r / std::f64::consts::TAU;
    if t < 0.0 { t += 1.0; }
    Ok(vec![num(t)])
}
fn api_max(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let mut r = arg_num(&a, 0).unwrap_or(0.0);
    for j in 1..a.len() { r = r.max(a[j].as_number().unwrap_or(0.0)); }
    Ok(vec![num(r)])
}
fn api_min(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let mut r = arg_num(&a, 0).unwrap_or(0.0);
    for j in 1..a.len() { r = r.min(a[j].as_number().unwrap_or(0.0)); }
    Ok(vec![num(r)])
}
fn api_mid(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = arg_num(&a, 0).unwrap_or(0.0);
    let y = arg_num(&a, 1).unwrap_or(0.0);
    let z = arg_num(&a, 2).unwrap_or(0.0);
    Ok(vec![num(x.min(y).max(x.max(y).min(z)))])
}
fn api_rnd(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let Some(Value::Table(t)) = a.get(0) {
        let len = t.borrow().raw_len();
        if len == 0 { return Ok(vec![nil()]); }
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
    if host.rng_state == 0 { host.rng_state = 0x12345678; }
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
    Ok(vec![num(from_fixed(to_fixed(arg_num(&a, 0).unwrap_or(0.0)) & to_fixed(arg_num(&a, 1).unwrap_or(0.0))))])
}
fn api_bor(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(from_fixed(to_fixed(arg_num(&a, 0).unwrap_or(0.0)) | to_fixed(arg_num(&a, 1).unwrap_or(0.0))))])
}
fn api_bxor(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(from_fixed(to_fixed(arg_num(&a, 0).unwrap_or(0.0)) ^ to_fixed(arg_num(&a, 1).unwrap_or(0.0))))])
}
fn api_bnot(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(from_fixed(!to_fixed(arg_num(&a, 0).unwrap_or(0.0))))])
}
fn api_shl(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = to_fixed(arg_num(&a, 0).unwrap_or(0.0));
    let b = arg_int(&a, 1).unwrap_or(0);
    let r = if b >= 0 { x.wrapping_shl((b.min(31)) as u32) } else { x.wrapping_shr(((-b).min(31)) as u32) };
    Ok(vec![num(from_fixed(r))])
}
fn api_shr(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = to_fixed(arg_num(&a, 0).unwrap_or(0.0));
    let b = arg_int(&a, 1).unwrap_or(0);
    let r = if b >= 0 { x.wrapping_shr((b.min(31)) as u32) } else { x.wrapping_shl(((-b).min(31)) as u32) };
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

fn api_tostr(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = a.get(0).cloned().unwrap_or(Value::Nil);
    let s = match &v {
        Value::Nil => "[nil]".to_string(),
        Value::Bool(b) => if *b { "true".into() } else { "false".into() },
        Value::Number(n) => number_to_str(*n),
        Value::Str(b) => return Ok(vec![Value::Str(Rc::clone(b))]),
        Value::Table(_) => "[table]".to_string(),
        Value::Function(_) => "[function]".to_string(),
    };
    Ok(vec![str_v(s.as_bytes())])
}

fn api_tonum(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let v = a.get(0).cloned().unwrap_or(Value::Nil);
    Ok(vec![match v.as_number() { Some(n) => num(n), None => nil() }])
}

fn api_sub(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let s = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    let len = s.len() as i64;
    let start_raw = opt_int(&a, 1, 1) as i64;
    let end_raw = opt_int(&a, 2, len as i32) as i64;
    let start = if start_raw < 0 { (len + start_raw + 1).max(1) } else { start_raw.max(1) };
    let end = if end_raw < 0 { len + end_raw + 1 } else { end_raw.min(len) };
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
            let n = n as i32;
            if n >= 0 && n <= 255 { out.push(n as u8); }
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

fn api_split(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let s = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    let convert = if a.len() <= 2 || matches!(a.get(2), Some(Value::Nil)) { true } else { a[2].truthy() };
    let t = Rc::new(RefCell::new(TableInner::new()));

    if let Some(Value::Number(n)) = a.get(1) {
        let n = if n.is_nan() || *n < 1.0 { 1usize } else if *n > 255.0 { 255 } else { *n as usize };
        let mut idx: i64 = 1;
        let mut pos = 0usize;
        while pos < s.len() {
            let end = (pos + n).min(s.len());
            let part = &s[pos..end];
            let v = if convert {
                match std::str::from_utf8(part).ok().and_then(|x| x.parse::<f64>().ok()) {
                    Some(f) => num(f),
                    None => Value::Str(Rc::from(part)),
                }
            } else {
                Value::Str(Rc::from(part))
            };
            t.borrow_mut().set(num(idx as f64), v);
            idx += 1; pos = end;
        }
        return Ok(vec![Value::Table(t)]);
    }

    let sep = match a.get(1) {
        Some(v) => v.as_str().unwrap_or_else(|| Rc::from(b",".as_slice())),
        None => Rc::from(b",".as_slice()),
    };
    if sep.is_empty() {
        let mut idx: i64 = 1;
        for &c in s.iter() {
            t.borrow_mut().set(num(idx as f64), Value::Str(Rc::from(&[c][..])));
            idx += 1;
        }
        return Ok(vec![Value::Table(t)]);
    }
    let mut idx: i64 = 1;
    let mut start = 0usize;
    let mut i = 0usize;
    while i + sep.len() <= s.len() {
        if &s[i..i + sep.len()] == &sep[..] {
            let part = &s[start..i];
            let v = if convert {
                match std::str::from_utf8(part).ok().and_then(|x| x.parse::<f64>().ok()) {
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
        match std::str::from_utf8(part).ok().and_then(|x| x.parse::<f64>().ok()) {
            Some(f) => num(f),
            None => Value::Str(Rc::from(part)),
        }
    } else {
        Value::Str(Rc::from(part))
    };
    t.borrow_mut().set(num(idx as f64), v);
    Ok(vec![Value::Table(t)])
}

fn str_len(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let s = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    Ok(vec![num(s.len() as f64)])
}
fn str_lower(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let s = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    let r: Vec<u8> = s.iter().map(|&c| c.to_ascii_lowercase()).collect();
    Ok(vec![Value::Str(Rc::from(r.as_slice()))])
}
fn str_upper(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let s = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    let r: Vec<u8> = s.iter().map(|&c| c.to_ascii_uppercase()).collect();
    Ok(vec![Value::Str(Rc::from(r.as_slice()))])
}
fn str_rep(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let s = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    let n = opt_int(&a, 1, 0).max(0) as usize;
    let mut r = Vec::with_capacity(s.len() * n);
    for _ in 0..n { r.extend_from_slice(&s); }
    Ok(vec![Value::Str(Rc::from(r.as_slice()))])
}
fn str_format(_i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    // Minimal %d, %s, %f, %x, %% support
    let fmt = arg_str(&a, 0).unwrap_or_else(|| Rc::from(&[][..]));
    let mut out = Vec::new();
    let mut argi = 1usize;
    let mut i = 0;
    while i < fmt.len() {
        let c = fmt[i];
        if c == b'%' && i + 1 < fmt.len() {
            i += 1;
            match fmt[i] {
                b'%' => out.push(b'%'),
                b'd' => {
                    let n = arg_num(&a, argi).unwrap_or(0.0) as i64;
                    out.extend_from_slice(format!("{}", n).as_bytes());
                    argi += 1;
                }
                b's' => {
                    let s = a.get(argi).and_then(|v| v.as_str()).unwrap_or_else(|| Rc::from(&[][..]));
                    out.extend_from_slice(&s);
                    argi += 1;
                }
                b'f' => {
                    let n = arg_num(&a, argi).unwrap_or(0.0);
                    out.extend_from_slice(format!("{}", n).as_bytes());
                    argi += 1;
                }
                b'x' => {
                    let n = arg_num(&a, argi).unwrap_or(0.0) as i64;
                    out.extend_from_slice(format!("{:x}", n).as_bytes());
                    argi += 1;
                }
                _ => out.push(c),
            }
            i += 1;
        } else {
            out.push(c);
            i += 1;
        }
    }
    Ok(vec![Value::Str(Rc::from(out.as_slice()))])
}

// === Table ===

fn api_add(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let Some(Value::Table(t)) = args.get(0) {
        let v = args.get(1).cloned().unwrap_or(Value::Nil);
        let pos = args.get(2).and_then(|x| x.as_number()).map(|n| n as i64);
        let mut tb = t.borrow_mut();
        let len = tb.raw_len();
        if let Some(p) = pos {
            if p < 1 { return Ok(vec![v]); }
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
    if let (Some(Value::Table(t)), Some(target)) = (args.get(0), args.get(1)) {
        let mut tb = t.borrow_mut();
        let len = tb.raw_len();
        let mut found_idx: Option<i64> = None;
        for i in 1..=len {
            let v = tb.map.get(&Key::Int(i)).cloned().unwrap_or(Value::Nil);
            if v.raw_equal(target) { found_idx = Some(i); break; }
        }
        if let Some(i) = found_idx {
            let removed = tb.map.get(&Key::Int(i)).cloned().unwrap_or(Value::Nil);
            for k in i..len {
                let v_ = tb.map.get(&Key::Int(k + 1)).cloned().unwrap_or(Value::Nil);
                if matches!(v_, Value::Nil) { tb.map.remove(&Key::Int(k)); }
                else { tb.map.insert(Key::Int(k), v_); }
            }
            tb.map.remove(&Key::Int(len));
            return Ok(vec![removed]);
        }
    }
    Ok(vec![Value::Nil])
}

fn api_deli(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    if let Some(Value::Table(t)) = args.get(0) {
        let mut tb = t.borrow_mut();
        let len = tb.raw_len();
        let idx = match args.get(1) {
            Some(v) if !matches!(v, Value::Nil) => v.as_number().unwrap_or(0.0) as i64,
            _ => len,
        };
        if idx < 1 || idx > len { return Ok(vec![Value::Nil]); }
        let removed = tb.map.get(&Key::Int(idx)).cloned().unwrap_or(Value::Nil);
        for k in idx..len {
            let v_ = tb.map.get(&Key::Int(k + 1)).cloned().unwrap_or(Value::Nil);
            if matches!(v_, Value::Nil) { tb.map.remove(&Key::Int(k)); }
            else { tb.map.insert(Key::Int(k), v_); }
        }
        tb.map.remove(&Key::Int(len));
        return Ok(vec![removed]);
    }
    Ok(vec![Value::Nil])
}

fn api_count(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = match args.get(0) { Some(Value::Table(t)) => t.clone(), _ => return Ok(vec![num(0.0)]) };
    if let Some(target) = args.get(1) {
        if !matches!(target, Value::Nil) {
            let tb = t.borrow();
            let len = tb.raw_len();
            let mut c = 0;
            for i in 1..=len {
                if let Some(v) = tb.map.get(&Key::Int(i)) {
                    if v.raw_equal(target) { c += 1; }
                }
            }
            return Ok(vec![num(c as f64)]);
        }
    }
    let n = t.borrow().raw_len() as f64;
    Ok(vec![num(n)])
}

fn api_foreach(i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = match args.get(0) { Some(Value::Table(t)) => t.clone(), _ => return Ok(vec![]) };
    let f = args.get(1).cloned().unwrap_or(Value::Nil);
    let mut idx: i64 = 1;
    loop {
        let len = t.borrow().raw_len();
        if idx > len { break; }
        let v = t.borrow().get(&num(idx as f64));
        let _ = i.call_value(&f, vec![v])?;
        let new_len = t.borrow().raw_len();
        if new_len >= len { idx += 1; }
    }
    Ok(vec![])
}

fn api_all(_i: &mut Interp, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let t = match args.get(0) {
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
    // ignore both — the captured state is the source of truth. This makes
    // us robust to PICO-Z's env-fallback `wrap` shim that re-wraps `all`.
    let state: Rc<RefCell<(Rc<Table>, i64, i64)>> =
        Rc::new(RefCell::new((Rc::clone(&t), 0i64, t.borrow().raw_len())));
    let iter = Value::Function(Function::Native(Rc::new(NativeFn {
        name: "all_iter",
        func: Box::new(move |_i, _args| {
            let mut st = state.borrow_mut();
            let tref = Rc::clone(&st.0);
            let len = tref.borrow().raw_len();
            // Only advance if no deletions occurred during the callback
            let next_idx = if len >= st.2 { st.1 + 1 } else { st.1 };
            if next_idx > len { return Ok(vec![nil()]); }
            st.1 = next_idx;
            st.2 = len;
            let v = tref.borrow().get(&num(next_idx as f64));
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
    let t = match args.get(0) { Some(Value::Table(t)) => t.clone(), _ => return Ok(vec![]) };
    let start = opt_int(&args, 1, 1) as i64;
    let end = if args.len() > 2 && !matches!(args[2], Value::Nil) {
        args[2].as_number().unwrap_or(0.0) as i64
    } else {
        let n = t.borrow().get(&str_v(b"n"));
        if let Some(nv) = n.as_number() { nv as i64 } else { t.borrow().raw_len() }
    };
    let mut out = Vec::new();
    for i in start..=end {
        out.push(t.borrow().get(&num(i as f64)));
    }
    Ok(out)
}

// === Graphics ===

fn api_cls(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = if a.is_empty() || matches!(a[0], Value::Nil) { 0 } else { (arg_int(&a, 0).unwrap_or(0) & 0xF) as u8 };
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
    if x < 0 || x >= 128 || y < 0 || y >= 128 { return Ok(vec![num(0.0)]); }
    let c = i.host().memory.screen_get(x as u8, y as u8);
    Ok(vec![num(c as f64)])
}
fn api_line(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    if a.is_empty() { st.line_valid = false; return Ok(vec![]); }
    if a.len() <= 3 || matches!(a.get(3), Some(Value::Nil)) {
        let x1 = arg_int(&a, 0).unwrap_or(0);
        let y1 = arg_int(&a, 1).unwrap_or(0);
        let col = get_color(st, &a, 2);
        if st.line_valid {
            gfx::draw_line(&mut st.memory, st.line_x, st.line_y, x1, y1, col);
        }
        st.line_x = x1; st.line_y = y1; st.line_valid = true;
        return Ok(vec![]);
    }
    let x0 = arg_int(&a, 0).unwrap_or(0);
    let y0 = arg_int(&a, 1).unwrap_or(0);
    let x1 = arg_int(&a, 2).unwrap_or(0);
    let y1 = arg_int(&a, 3).unwrap_or(0);
    let col = get_color(st, &a, 4);
    gfx::draw_line(&mut st.memory, x0, y0, x1, y1, col);
    st.line_x = x1; st.line_y = y1; st.line_valid = true;
    Ok(vec![])
}
fn api_rect(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 4);
    gfx::rect(&mut i.host().memory,
              arg_int(&a, 0).unwrap_or(0), arg_int(&a, 1).unwrap_or(0),
              arg_int(&a, 2).unwrap_or(0), arg_int(&a, 3).unwrap_or(0), col);
    Ok(vec![])
}
fn api_rectfill(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 4);
    gfx::rectfill(&mut i.host().memory,
                  arg_int(&a, 0).unwrap_or(0), arg_int(&a, 1).unwrap_or(0),
                  arg_int(&a, 2).unwrap_or(0), arg_int(&a, 3).unwrap_or(0), col);
    Ok(vec![])
}
fn api_circ(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 3);
    gfx::circ(&mut i.host().memory,
              arg_int(&a, 0).unwrap_or(0), arg_int(&a, 1).unwrap_or(0), opt_int(&a, 2, 4), col);
    Ok(vec![])
}
fn api_circfill(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 3);
    gfx::circfill(&mut i.host().memory,
                  arg_int(&a, 0).unwrap_or(0), arg_int(&a, 1).unwrap_or(0), opt_int(&a, 2, 4), col);
    Ok(vec![])
}
fn api_oval(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 4);
    gfx::oval(&mut i.host().memory,
              arg_int(&a, 0).unwrap_or(0), arg_int(&a, 1).unwrap_or(0),
              arg_int(&a, 2).unwrap_or(0), arg_int(&a, 3).unwrap_or(0), col);
    Ok(vec![])
}
fn api_ovalfill(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let col = get_color(i.host(), &a, 4);
    gfx::ovalfill(&mut i.host().memory,
                  arg_int(&a, 0).unwrap_or(0), arg_int(&a, 1).unwrap_or(0),
                  arg_int(&a, 2).unwrap_or(0), arg_int(&a, 3).unwrap_or(0), col);
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
    if x < 0 || x >= 128 || y < 0 || y >= 128 { return Ok(vec![num(0.0)]); }
    Ok(vec![num(i.host().memory.sprite_get(x as u8, y as u8) as f64)])
}
fn api_sset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let x = arg_int(&a, 0).unwrap_or(0);
    let y = arg_int(&a, 1).unwrap_or(0);
    let col = get_color(i.host(), &a, 2);
    if x >= 0 && x < 128 && y >= 0 && y < 128 {
        i.host().memory.sprite_set(x as u8, y as u8, col);
    }
    Ok(vec![])
}
fn api_fget(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let n = arg_int(&a, 0).unwrap_or(0);
    if n < 0 || n >= 256 { return Ok(vec![num(0.0)]); }
    let flags = i.host().memory.ram[mem::ADDR_FLAGS as usize + n as usize];
    if a.len() <= 1 || matches!(a[1], Value::Nil) {
        return Ok(vec![num(flags as f64)]);
    }
    let bit = arg_int(&a, 1).unwrap_or(0) as u32 & 7;
    Ok(vec![boolv(flags & (1u8 << bit) != 0)])
}
fn api_fset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let n = arg_int(&a, 0).unwrap_or(0);
    if n < 0 || n >= 256 { return Ok(vec![]); }
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
    let txt = a.get(0).cloned().map(|v| v.as_str()).flatten()
        .unwrap_or_else(|| Rc::from(b"".as_slice()));
    let st = i.host();
    let r = if a.len() <= 1 || matches!(a.get(1), Some(Value::Nil)) {
        let cx = st.memory.ram[mem::ADDR_CURSOR_X as usize] as i32;
        let cy = st.memory.ram[mem::ADDR_CURSOR_Y as usize] as i32;
        let col = st.memory.ram[mem::ADDR_COLOR as usize] & 0x0F;
        let r = gfx::draw_text(&mut st.memory, &txt, cx, cy, col);
        st.memory.ram[mem::ADDR_CURSOR_Y as usize] = ((cy + 6) & 0xFF) as u8;
        r
    } else if a.len() <= 2 || matches!(a.get(2), Some(Value::Nil)) {
        let cx = st.memory.ram[mem::ADDR_CURSOR_X as usize] as i32;
        let cy = st.memory.ram[mem::ADDR_CURSOR_Y as usize] as i32;
        let col = get_color(st, &a, 1);
        let r = gfx::draw_text(&mut st.memory, &txt, cx, cy, col);
        st.memory.ram[mem::ADDR_CURSOR_Y as usize] = ((cy + 6) & 0xFF) as u8;
        r
    } else {
        let x = arg_int(&a, 1).unwrap_or(0);
        let y = arg_int(&a, 2).unwrap_or(0);
        let col = get_color(st, &a, 3);
        gfx::draw_text(&mut st.memory, &txt, x, y, col)
    };
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
    Ok(vec![num(prev_x as f64), num(prev_y as f64), num(prev_w as f64), num(prev_h as f64)])
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
    if let Some(Value::Table(t)) = a.get(0) {
        let p = opt_int(&a, 1, 0);
        for k in 0..16 {
            let v = t.borrow().get(&num(k as f64));
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
            if bits & (1 << k) != 0 {
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
    st.memory.poke16(mem::ADDR_FILL_PAT, (int_val & 0xFFFF) as u16);
    let trans = if int_val & 0x10000 != 0 { 1 } else { 0 };
    st.memory.ram[mem::ADDR_FILL_PAT as usize + 2] = trans;
    Ok(vec![])
}

// === Input ===

fn api_btn(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    if a.is_empty() || matches!(a[0], Value::Nil) {
        let p0 = (st.input.btn_state[0] & 0x3F) as u32;
        let p1 = ((st.input.btn_state[1] & 0x3F) as u32) << 8;
        return Ok(vec![num((p0 | p1) as f64)]);
    }
    let b = (arg_int(&a, 0).unwrap_or(0) as u8) & 7;
    let p = (opt_int(&a, 1, 0) as u8) & 1;
    Ok(vec![boolv(st.input.btn(b, p))])
}
fn api_btnp(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let st = i.host();
    if a.is_empty() || matches!(a[0], Value::Nil) {
        let mut p0: u32 = 0;
        let mut p1: u32 = 0;
        for b in 0..6u8 {
            if st.input.btnp(b, 0, &st.memory) { p0 |= 1 << b; }
            if st.input.btnp(b, 1, &st.memory) { p1 |= 1 << b; }
        }
        return Ok(vec![num((p0 | (p1 << 8)) as f64)]);
    }
    let b = (arg_int(&a, 0).unwrap_or(0) as u8) & 7;
    let p = (opt_int(&a, 1, 0) as u8) & 1;
    Ok(vec![boolv(st.input.btnp(b, p, &st.memory))])
}

// === Memory ===

fn api_peek(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_int(&a, 0).unwrap_or(0) as u16;
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
    let addr = arg_int(&a, 0).unwrap_or(0) as u16;
    for (k, v) in a.iter().enumerate().skip(1) {
        let val = (v.as_number().unwrap_or(0.0) as i32 & 0xFF) as u8;
        i.host().memory.poke(addr.wrapping_add((k - 1) as u16), val);
    }
    Ok(vec![])
}
fn api_peek2(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_int(&a, 0).unwrap_or(0) as u16;
    Ok(vec![num(i.host().memory.peek16(addr) as f64)])
}
fn api_poke2(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_int(&a, 0).unwrap_or(0) as u16;
    let val = (safe_to_i32(arg_num(&a, 1).unwrap_or(0.0)) as u32 & 0xFFFF) as u16;
    i.host().memory.poke16(addr, val);
    Ok(vec![])
}
fn api_peek4(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_int(&a, 0).unwrap_or(0) as u16;
    let raw = i.host().memory.peek32(addr);
    let fixed = raw as i32;
    Ok(vec![num(fixed as f64 / 65536.0)])
}
fn api_poke4(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let addr = arg_int(&a, 0).unwrap_or(0) as u16;
    let v = arg_num(&a, 1).unwrap_or(0.0);
    let scaled = v * 65536.0;
    let fixed = if scaled >= i32::MAX as f64 { i32::MAX }
                else if scaled <= i32::MIN as f64 { i32::MIN }
                else { scaled as i32 };
    i.host().memory.poke32(addr, fixed as u32);
    Ok(vec![])
}
fn api_memcpy(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    i.host().memory.memcpy(arg_int(&a, 0).unwrap_or(0) as u16,
                            arg_int(&a, 1).unwrap_or(0) as u16,
                            arg_int(&a, 2).unwrap_or(0) as u16);
    Ok(vec![])
}
fn api_memset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    i.host().memory.memset(arg_int(&a, 0).unwrap_or(0) as u16,
                            (arg_int(&a, 1).unwrap_or(0) & 0xFF) as u8,
                            arg_int(&a, 2).unwrap_or(0) as u16);
    Ok(vec![])
}
fn api_reload(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let dst = if a.is_empty() || matches!(a[0], Value::Nil) { 0 } else { arg_int(&a, 0).unwrap_or(0) as u16 };
    let src = if a.len() < 2 || matches!(a[1], Value::Nil) { 0 } else { arg_int(&a, 1).unwrap_or(0) as u16 };
    let len = if a.len() < 3 || matches!(a[2], Value::Nil) { 0x4300 } else { arg_int(&a, 2).unwrap_or(0) as u16 };
    i.host().memory.reload(dst, src, len);
    Ok(vec![])
}
fn api_cstore(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let dst = if a.is_empty() || matches!(a[0], Value::Nil) { 0 } else { arg_int(&a, 0).unwrap_or(0) as u16 };
    let src = if a.len() < 2 || matches!(a[1], Value::Nil) { 0 } else { arg_int(&a, 1).unwrap_or(0) as u16 };
    let len = if a.len() < 3 || matches!(a[2], Value::Nil) { 0x4300 } else { arg_int(&a, 2).unwrap_or(0) as u16 };
    let st = i.host();
    for k in 0..len { st.memory.rom[dst.wrapping_add(k) as usize] = st.memory.ram[src.wrapping_add(k) as usize]; }
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
    let mask = opt_int(&a, 2, 0);
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
        16..=19 => {
            let c = &st.audio.channels[(n - 16) as usize];
            if !c.finished && c.sfx_id >= 0 { num(c.note_index as f64) } else { num(0.0) }
        }
        20..=23 => {
            let c = &st.audio.channels[(n - 20) as usize];
            if !c.finished && c.sfx_id >= 0 { num(c.sfx_id as f64) } else { num(-1.0) }
        }
        24 => num(st.audio.music_state.pattern as f64),
        26 => num(st.audio.music_state.tick as f64),
        30 => boolv(st.input.key_chars_len > 0),
        31 => str_v(b""),
        32 => num(if st.memory.ram[0x5F2D] & 1 != 0 { st.input.mouse_x as f64 } else { 0.0 }),
        33 => num(if st.memory.ram[0x5F2D] & 1 != 0 { st.input.mouse_y as f64 } else { 0.0 }),
        34 => num(if st.memory.ram[0x5F2D] & 1 != 0 { st.input.mouse_buttons as f64 } else { 0.0 }),
        36 => num(if st.memory.ram[0x5F2D] & 1 != 0 { st.input.mouse_wheel as f64 } else { 0.0 }),
        46..=49 => {
            let c = &st.audio.channels[(n - 46) as usize];
            if !c.finished && c.sfx_id >= 0 { num(c.note_index as f64) } else { num(0.0) }
        }
        50..=53 => {
            let c = &st.audio.channels[(n - 50) as usize];
            if !c.finished && c.sfx_id >= 0 { num(c.sfx_id as f64) } else { num(-1.0) }
        }
        54 => num(st.audio.music_state.pattern as f64),
        55 => num(st.audio.music_state.total_patterns as f64),
        56 => num(st.audio.music_state.tick as f64),
        57 => boolv(st.audio.music_state.playing),
        _ => num(0.0),
    };
    Ok(vec![v])
}
fn api_time(i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![num(i.host().elapsed_time)])
}
fn api_printh(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![])
}
fn api_cartdata(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    Ok(vec![boolv(true)])
}
fn api_dget(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let idx = arg_int(&a, 0).unwrap_or(0);
    if idx < 0 || idx >= 64 { return Ok(vec![num(0.0)]); }
    let addr = mem::ADDR_CART_DATA + (idx as u16) * 4;
    let raw = i.host().memory.peek32(addr);
    let fixed = raw as i32;
    Ok(vec![num(fixed as f64 / 65536.0)])
}
fn api_dset(i: &mut Interp, a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    let idx = arg_int(&a, 0).unwrap_or(0);
    let v = arg_num(&a, 1).unwrap_or(0.0);
    if idx < 0 || idx >= 64 { return Ok(vec![]); }
    let addr = mem::ADDR_CART_DATA + (idx as u16) * 4;
    let scaled = v * 65536.0;
    let fixed = if scaled >= i32::MAX as f64 { i32::MAX }
                else if scaled <= i32::MIN as f64 { i32::MIN }
                else { scaled as i32 };
    i.host().memory.poke32(addr, fixed as u32);
    Ok(vec![])
}
fn api_menuitem(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> { Ok(vec![]) }
fn api_extcmd(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> { Ok(vec![]) }
fn api_flip(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> { Ok(vec![]) }
fn api_reset(i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> {
    i.host().memory.init_draw_state();
    Ok(vec![])
}
fn api_stop(_i: &mut Interp, _a: Vec<Value>) -> Result<Vec<Value>, RtError> { Ok(vec![]) }
