// Value, Table, Function. Single-threaded; uses Rc/RefCell for shared mutable
// state. Hashing/equality follow Lua semantics:
//   - numbers: bit-identical for hashing; integral floats compare equal to
//     same integer (we treat all numbers as f64).
//   - strings: byte-equal.
//   - tables/functions: identity (Rc pointer).

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::ast::FuncBody;
use super::interp::Interp;

pub type Table = RefCell<TableInner>;

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{:?}", String::from_utf8_lossy(s)),
            Value::Table(t) => write!(f, "table:0x{:x}", Rc::as_ptr(t) as usize),
            Value::Function(g) => write!(f, "{:?}", g),
        }
    }
}

/// Insertion-ordered map with a HashMap-compatible surface. PICO-8's
/// `pairs()`/`next()` iteration order is deterministic on real hardware
/// (Lua's internal order, stable per table build); a randomized per-process
/// HashMap order made carts whose logic depends on iteration order fail
/// NONDETERMINISTICALLY (samurise-1.p8.png's embedded LISP VM failed with a
/// different error on every run). Entries keep insertion order; removals
/// leave tombstones that are compacted when they outnumber live entries.
pub struct OrderedMap {
    idx: HashMap<Key, usize>,
    entries: Vec<Option<(Key, Value)>>,
    live: usize,
}

impl OrderedMap {
    pub fn new() -> Self {
        Self {
            idx: HashMap::new(),
            entries: Vec::new(),
            live: 0,
        }
    }
    pub fn get(&self, k: &Key) -> Option<&Value> {
        let &i = self.idx.get(k)?;
        self.entries[i].as_ref().map(|(_, v)| v)
    }
    pub fn contains_key(&self, k: &Key) -> bool {
        self.idx.contains_key(k)
    }
    pub fn insert(&mut self, k: Key, v: Value) -> Option<Value> {
        if let Some(&i) = self.idx.get(&k) {
            let slot = self.entries[i].as_mut().unwrap();
            return Some(std::mem::replace(&mut slot.1, v));
        }
        self.idx.insert(k.clone(), self.entries.len());
        self.entries.push(Some((k, v)));
        self.live += 1;
        None
    }
    pub fn remove(&mut self, k: &Key) -> Option<Value> {
        let i = self.idx.remove(k)?;
        let (_, v) = self.entries[i].take()?;
        self.live -= 1;
        if self.entries.len() > 8 && self.live * 2 < self.entries.len() {
            self.compact();
        }
        Some(v)
    }
    fn compact(&mut self) {
        let old = std::mem::take(&mut self.entries);
        self.idx.clear();
        for e in old.into_iter().flatten() {
            self.idx.insert(e.0.clone(), self.entries.len());
            self.entries.push(Some(e));
        }
    }
    pub fn clear(&mut self) {
        self.idx.clear();
        self.entries.clear();
        self.live = 0;
    }
    pub fn len(&self) -> usize {
        self.live
    }
    pub fn is_empty(&self) -> bool {
        self.live == 0
    }
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.entries.iter().flatten().map(|(k, _)| k)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Value)> {
        self.entries.iter().flatten().map(|(k, v)| (k, v))
    }
}

impl Default for OrderedMap {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TableInner {
    pub map: OrderedMap,
    /// Tracks the largest sequential integer key written, for `#t`.
    /// We rebuild lazily.
    pub array_max_hint: u32,
    /// Positional span declared by a table constructor, COUNTING nil
    /// slots (`{nil,nil,49}` declares 3). Real Lua pre-sizes the array
    /// part to this, which is what makes `#{...}` from varargs with
    /// leading nils report the full span -- the border search consults
    /// this the way luaH_getn consults the array-part size.
    pub array_decl: i64,
    pub metatable: Option<Rc<Table>>,
}

impl Default for TableInner {
    fn default() -> Self {
        Self::new()
    }
}

impl TableInner {
    pub fn new() -> Self {
        Self {
            map: OrderedMap::new(),
            array_max_hint: 0,
            array_decl: 0,
            metatable: None,
        }
    }
    pub fn get(&self, k: &Value) -> Value {
        if matches!(k, Value::Nil) {
            return Value::Nil;
        }
        let key = match Key::from_value(k) {
            Some(k) => k,
            None => return Value::Nil,
        };
        self.map.get(&key).cloned().unwrap_or(Value::Nil)
    }
    pub fn set(&mut self, k: Value, v: Value) {
        if matches!(k, Value::Nil) {
            return;
        }
        let key = match Key::from_value(&k) {
            Some(k) => k,
            None => return,
        };
        if let Key::Int(i) = key {
            if i > 0 && (i as u32) > self.array_max_hint {
                self.array_max_hint = i as u32;
            }
        }
        if matches!(v, Value::Nil) {
            self.map.remove(&key);
        } else {
            self.map.insert(key, v);
        }
    }
    pub fn raw_len(&self) -> i64 {
        // Official PICO-8's `#` (and unpack(), which uses it) follows
        // Lua's luaH_getn border search, which SPANS interior nil holes:
        // `#{1,2,nil,4,5}` is 5, sparse `u[1] u[2] u[4] u[5]` is 5, but a
        // lone `u[1000]=1` is 0 and `{1,2,3}` plus `w[1000]=1` is 3 (all
        // five shapes oracle-confirmed; a real corpus cart destructures
        // containers holding nil fields via `unpack(self)`,
        // fakogejuzo-0.p8.png). Doubling probe then binary search for a
        // border t[i]~=nil, t[i+1]==nil.
        let has = |i: i64| self.map.contains_key(&Key::Int(i));
        // Constructor-declared span first (see `array_decl`): if the last
        // declared slot is filled, the border is at least there (keep
        // searching upward for appended elements); if it's a nil hole,
        // binary-search the border inside the declared span -- exactly
        // luaH_getn's array-part handling.
        if self.array_decl > 0 {
            if !has(self.array_decl) {
                let mut i: i64 = 0;
                let mut j: i64 = self.array_decl;
                while j - i > 1 {
                    let m = (i + j) / 2;
                    if has(m) {
                        i = m;
                    } else {
                        j = m;
                    }
                }
                return i;
            }
            // fall through with the search starting at the declared end
            let mut i: i64 = self.array_decl;
            let mut j: i64 = self.array_decl + 1;
            while has(j) {
                i = j;
                if j > i64::MAX / 2 {
                    break;
                }
                j *= 2;
            }
            while j - i > 1 {
                let m = (i + j) / 2;
                if has(m) {
                    i = m;
                } else {
                    j = m;
                }
            }
            return i;
        }
        if !has(1) {
            return 0;
        }
        let mut i: i64 = 1;
        let mut j: i64 = 2;
        while has(j) {
            i = j;
            if j > i64::MAX / 2 {
                break;
            }
            j *= 2;
        }
        // Border in (i, j]: t[i] ~= nil, t[j] == nil.
        while j - i > 1 {
            let m = (i + j) / 2;
            if has(m) {
                i = m;
            } else {
                j = m;
            }
        }
        i
    }
}

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Str(Rc<[u8]>),
    Table(Rc<Table>),
    Function(Function),
}

impl Value {
    pub fn truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::Str(_) => "string",
            Value::Table(_) => "table",
            Value::Function(_) => "function",
        }
    }
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Str(s) => {
                let s = std::str::from_utf8(s).ok()?.trim();
                if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                    i64::from_str_radix(rest, 16)
                        .ok()
                        .map(|n| n as f64)
                        .or_else(|| u64::from_str_radix(rest, 16).ok().map(|n| n as f64))
                } else {
                    s.parse::<f64>().ok()
                }
            }
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<Rc<[u8]>> {
        match self {
            Value::Str(s) => Some(Rc::clone(s)),
            Value::Number(n) => {
                let s = number_to_str(*n);
                Some(Rc::from(s.as_bytes()))
            }
            _ => None,
        }
    }
    pub fn raw_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Table(a), Value::Table(b)) => Rc::ptr_eq(a, b),
            (Value::Function(a), Value::Function(b)) => a.identity_eq(b),
            _ => false,
        }
    }
}

pub fn number_to_str(n: f64) -> String {
    if n.is_nan() {
        return "nan".into();
    }
    if n.is_infinite() {
        return if n > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if n == n.trunc() && n.abs() < 1e16 {
        return format!("{}", n as i64);
    }
    // PICO-8 numbers are 16.16 fixed point, displayed rounded to 4 decimal
    // digits with trailing zeros stripped (confirmed against official
    // PICO-8: e.g. 1/3 -> "0.3333", 2.99998 -> "3"). A small-magnitude
    // float that rounds to "0.0000" at this display precision shows "0",
    // not "-0" -- confirmed against official PICO-8 (`-0.00001` -> "0").
    // Note this collapse is technically over-broad: official PICO-8
    // genuinely prints "-0" for a handful of values that reach exactly the
    // smallest representable negative fixed-point increment via bit
    // manipulation (e.g. `bnot(0)`), which would need real 16.16
    // quantization of every literal/arithmetic result (a known, deferred
    // architectural gap -- see LEDGER.md) to distinguish correctly from an
    // ordinary tiny float like `-0.00001` that real hardware collapses to
    // exact zero at parse time. Until then this collapses both alike,
    // matching the far more common ordinary-tiny-float case.
    let mut s = format!("{:.4}", n);
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

/// Hashable key derived from a Value.
#[derive(Debug, Clone)]
pub enum Key {
    Bool(bool),
    Int(i64),
    Float(u64), // f64 bits, normalized for NaN
    Str(Rc<[u8]>),
    Table(usize), // Rc::as_ptr() as usize
    Function(usize),
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Key::Bool(a), Key::Bool(b)) => a == b,
            (Key::Int(a), Key::Int(b)) => a == b,
            (Key::Float(a), Key::Float(b)) => a == b,
            (Key::Int(a), Key::Float(b)) | (Key::Float(b), Key::Int(a)) => {
                let f = f64::from_bits(*b);
                f == *a as f64 && f.fract() == 0.0
            }
            (Key::Str(a), Key::Str(b)) => a == b,
            (Key::Table(a), Key::Table(b)) => a == b,
            (Key::Function(a), Key::Function(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Key {}
impl std::hash::Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Key::Bool(b) => {
                0u8.hash(state);
                b.hash(state);
            }
            // Both Int and Float-with-integer-value should hash the same.
            // Convert to canonical form: if float is integral and fits in i64, treat as Int.
            Key::Int(i) => {
                1u8.hash(state);
                i.hash(state);
            }
            Key::Float(bits) => {
                let f = f64::from_bits(*bits);
                if f.is_finite() && f == f.trunc() && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                    1u8.hash(state);
                    (f as i64).hash(state);
                } else {
                    2u8.hash(state);
                    bits.hash(state);
                }
            }
            Key::Str(s) => {
                3u8.hash(state);
                s.hash(state);
            }
            Key::Table(p) => {
                4u8.hash(state);
                p.hash(state);
            }
            Key::Function(p) => {
                5u8.hash(state);
                p.hash(state);
            }
        }
    }
}

impl Key {
    pub fn from_value(v: &Value) -> Option<Key> {
        match v {
            Value::Nil => None,
            Value::Bool(b) => Some(Key::Bool(*b)),
            Value::Number(n) => {
                if n.is_nan() {
                    return None;
                }
                if *n == n.trunc() && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                    Some(Key::Int(*n as i64))
                } else {
                    Some(Key::Float(n.to_bits()))
                }
            }
            Value::Str(s) => Some(Key::Str(Rc::clone(s))),
            Value::Table(t) => Some(Key::Table(Rc::as_ptr(t) as usize)),
            Value::Function(f) => Some(Key::Function(f.identity())),
        }
    }
}

/// Function values: native (Rust) or Lua closure.
#[derive(Clone)]
pub enum Function {
    Native(Rc<NativeFn>),
    Lua(Rc<LuaClosure>),
}

pub struct NativeFn {
    pub name: &'static str,
    /// Native callbacks receive the interpreter (for invoking the engine,
    /// allocating tables, error-raising) and the args. They return values.
    pub func: Box<NativeCallback>,
}

pub type NativeCallback = dyn Fn(&mut Interp, Vec<Value>) -> Result<Vec<Value>, RtError>;

pub struct LuaClosure {
    pub body: Rc<FuncBody>,
    pub upvalues: Vec<Rc<RefCell<Value>>>,
    pub upnames: Vec<Rc<str>>,
}

impl Function {
    pub fn identity(&self) -> usize {
        match self {
            Function::Native(n) => Rc::as_ptr(n) as usize,
            Function::Lua(c) => Rc::as_ptr(c) as usize,
        }
    }
    pub fn identity_eq(&self, other: &Function) -> bool {
        self.identity() == other.identity()
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native(n) => write!(f, "native({})", n.name),
            Function::Lua(_) => write!(f, "lua-closure"),
        }
    }
}

/// Deep-clone a value tree. Tables are cloned with cycle detection; functions
/// (Rc-shared) are kept by reference — they're code, not state. Used by the
/// in-memory save-state snapshot to capture mutable Lua state without going
/// through serialization.
pub fn deep_clone_value(
    v: &Value,
    seen: &mut std::collections::HashMap<usize, Rc<Table>>,
) -> Value {
    match v {
        Value::Table(t) => Value::Table(deep_clone_table(t, seen)),
        other => other.clone(),
    }
}

pub fn deep_clone_table(
    t: &Rc<Table>,
    seen: &mut std::collections::HashMap<usize, Rc<Table>>,
) -> Rc<Table> {
    let ptr = Rc::as_ptr(t) as usize;
    if let Some(existing) = seen.get(&ptr) {
        return Rc::clone(existing);
    }
    let new = Rc::new(RefCell::new(TableInner::new()));
    seen.insert(ptr, Rc::clone(&new));
    {
        let src = t.borrow();
        let mut dst = new.borrow_mut();
        dst.metatable = src.metatable.as_ref().map(|m| deep_clone_table(m, seen));
        dst.array_max_hint = src.array_max_hint;
        // Build the cloned map; collect first to release the src borrow before recursing
        let pairs: Vec<(Key, Value)> = src
            .map
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        drop(src);
        drop(dst);
        for (k, v) in pairs {
            let nv = deep_clone_value(&v, seen);
            new.borrow_mut().map.insert(k, nv);
        }
    }
    new
}

/// Runtime error carrying a value (Lua errors can be any value).
pub struct RtError {
    pub value: Value,
}

impl RtError {
    pub fn msg(s: impl Into<String>) -> Self {
        let s: String = s.into();
        RtError {
            value: Value::Str(Rc::from(s.as_bytes())),
        }
    }
}
