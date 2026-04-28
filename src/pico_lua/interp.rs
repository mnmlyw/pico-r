// Tree-walking interpreter. Single-threaded; PICO-8 carts call into _init,
// _update, _draw — no coroutines required for the dominant cart pattern.
//
// The interpreter holds a pointer to the host PicoState (set by the engine
// before each call). Native callbacks dereference this to access memory,
// gfx, audio, etc.

use std::cell::RefCell;
use std::rc::Rc;

use super::ast::*;
use super::value::*;

use crate::state::PicoState;

/// Outcome of executing a block of statements.
pub enum Flow {
    Normal,
    Return(Vec<Value>),
    Break,
    Goto(Rc<str>),
}

pub struct Interp {
    pub globals: Rc<Table>,
    /// Lexical scope chain — innermost frame at the end. Each frame is a
    /// Vec of (name, slot) where slot is shared so closures can capture.
    pub frames: Vec<Frame>,
    /// `&mut PicoState` provided by the host while a Lua call is active.
    /// Stored as a raw pointer because callbacks need access without
    /// threading lifetimes through every frame.
    pub host: *mut PicoState,
    /// Source name (for error messages).
    pub chunkname: String,
    /// Detect runaway loops (e.g. cart with infinite while in _update).
    pub instruction_budget: u32,
    /// Most recent line number visited by the interpreter — used to annotate errors.
    pub current_line: u32,
}

pub struct Frame {
    pub locals: Vec<(Rc<str>, Rc<RefCell<Value>>)>,
    pub varargs: Vec<Value>,
    /// Upvalues for the enclosing closure (when calling a Lua function).
    pub upvalues: Vec<(Rc<str>, Rc<RefCell<Value>>)>,
}

impl Interp {
    pub fn new(globals: Rc<Table>) -> Self {
        Self {
            globals,
            frames: Vec::new(),
            host: std::ptr::null_mut(),
            chunkname: "cart".into(),
            instruction_budget: 0,
            current_line: 0,
        }
    }

    /// Access the host PICO-8 state. Caller must guarantee `host` is set.
    pub fn host(&self) -> &mut PicoState {
        unsafe { &mut *self.host }
    }

    fn push_frame(&mut self) {
        self.frames.push(Frame {
            locals: Vec::new(),
            varargs: Vec::new(),
            upvalues: Vec::new(),
        });
    }
    fn pop_frame(&mut self) {
        self.frames.pop();
    }

    fn declare_local(&mut self, name: Rc<str>, val: Value) {
        let slot = Rc::new(RefCell::new(val));
        let frame = self.frames.last_mut().expect("no frame");
        frame.locals.push((name, slot));
    }

    fn resolve(&mut self, name: &str) -> Value {
        if let Some(slot) = self.find_slot(name) {
            return slot.borrow().clone();
        }
        self.globals.borrow().get(&Value::Str(Rc::from(name.as_bytes())))
    }

    fn find_slot(&self, name: &str) -> Option<Rc<RefCell<Value>>> {
        let frame = self.frames.last()?;
        for (n, slot) in frame.locals.iter().rev() {
            if &**n == name { return Some(Rc::clone(slot)); }
        }
        for (n, slot) in frame.upvalues.iter() {
            if &**n == name { return Some(Rc::clone(slot)); }
        }
        None
    }

    pub fn assign_name(&mut self, name: &str, val: Value) {
        if let Some(slot) = self.find_slot(name) {
            *slot.borrow_mut() = val;
        } else {
            self.globals.borrow_mut().set(Value::Str(Rc::from(name.as_bytes())), val);
        }
    }

    /// Execute a top-level chunk as `function(...) <body> end` and call it
    /// with no args (top-level varargs are empty).
    pub fn execute_chunk(&mut self, body: &Block) -> Result<(), RtError> {
        self.push_frame();
        let res = self.execute_block(body);
        self.pop_frame();
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Run a block, leaving any locals it declared in place on the frame.
    /// Used by `repeat..until` so the until-clause sees the body's locals.
    pub fn execute_block_keep_scope(&mut self, block: &Block) -> Result<Flow, RtError> {
        self.execute_block_inner(block, false)
    }

    pub fn execute_block(&mut self, block: &Block) -> Result<Flow, RtError> {
        self.execute_block_inner(block, true)
    }

    fn execute_block_inner(&mut self, block: &Block, pop_locals: bool) -> Result<Flow, RtError> {
        let mut i = 0;
        let saved_locals = self.frames.last().map(|f| f.locals.len()).unwrap_or(0);
        if block.line > 0 { self.current_line = block.line; }
        let result = loop {
            if i >= block.stats.len() { break Ok(Flow::Normal); }
            if let Some(&l) = block.stat_lines.get(i) { if l > 0 { self.current_line = l; } }
            let stat = &block.stats[i];
            match self.execute_statement(stat) {
                Ok(Flow::Normal) => { i += 1; }
                Ok(Flow::Goto(label)) => {
                    // Try to find label in this block (forward or backward)
                    let mut found = None;
                    for (idx, s) in block.stats.iter().enumerate() {
                        if let Stat::Label(n) = s {
                            if n == &label { found = Some(idx); break; }
                        }
                    }
                    if let Some(idx) = found {
                        // Restore locals to start-of-block (basic approach)
                        if let Some(frame) = self.frames.last_mut() {
                            frame.locals.truncate(saved_locals);
                        }
                        i = idx + 1;
                    } else {
                        break Ok(Flow::Goto(label));
                    }
                }
                other => break other,
            }
        };
        // Pop locals declared in this block (caller may opt out)
        if pop_locals {
            if let Some(frame) = self.frames.last_mut() {
                frame.locals.truncate(saved_locals);
            }
        }
        result
    }

    fn execute_statement(&mut self, stat: &Stat) -> Result<Flow, RtError> {
        match stat {
            Stat::Assign(lhs, rhs) => {
                let values = self.eval_exp_list(rhs, lhs.len())?;
                for (i, target) in lhs.iter().enumerate() {
                    let v = values.get(i).cloned().unwrap_or(Value::Nil);
                    self.assign_to(target, v)?;
                }
                Ok(Flow::Normal)
            }
            Stat::LocalAssign(names, rhs) => {
                let values = self.eval_exp_list(rhs, names.len())?;
                for (i, n) in names.iter().enumerate() {
                    let v = values.get(i).cloned().unwrap_or(Value::Nil);
                    self.declare_local(Rc::clone(n), v);
                }
                Ok(Flow::Normal)
            }
            Stat::Call(e) => {
                let _ = self.eval_call_expr(e, false)?;
                Ok(Flow::Normal)
            }
            Stat::Do(b) => self.execute_block(b),
            Stat::While(cond, body) => {
                loop {
                    let c = self.eval_expr_single(cond)?;
                    if !c.truthy() { return Ok(Flow::Normal); }
                    match self.execute_block(body)? {
                        Flow::Normal => {}
                        Flow::Break => return Ok(Flow::Normal),
                        other => return Ok(other),
                    }
                }
            }
            Stat::Repeat(body, cond) => {
                // Lua semantics: `until` condition is evaluated in the scope of
                // the body, so we must NOT pop the body's locals before checking.
                loop {
                    let saved_locals = self.frames.last().map(|f| f.locals.len()).unwrap_or(0);
                    let r = self.execute_block_keep_scope(body);
                    let flow = match r {
                        Ok(Flow::Normal) => Flow::Normal,
                        Ok(Flow::Break) => {
                            if let Some(f) = self.frames.last_mut() { f.locals.truncate(saved_locals); }
                            return Ok(Flow::Normal);
                        }
                        Ok(other) => {
                            if let Some(f) = self.frames.last_mut() { f.locals.truncate(saved_locals); }
                            return Ok(other);
                        }
                        Err(e) => {
                            if let Some(f) = self.frames.last_mut() { f.locals.truncate(saved_locals); }
                            return Err(e);
                        }
                    };
                    let _ = flow;
                    let c = self.eval_expr_single(cond)?;
                    // Now pop the body's locals
                    if let Some(f) = self.frames.last_mut() { f.locals.truncate(saved_locals); }
                    if c.truthy() { return Ok(Flow::Normal); }
                }
            }
            Stat::If(arms, else_b) => {
                for (cond, body) in arms {
                    let c = self.eval_expr_single(cond)?;
                    if c.truthy() {
                        return self.execute_block(body);
                    }
                }
                if let Some(b) = else_b {
                    return self.execute_block(b);
                }
                Ok(Flow::Normal)
            }
            Stat::NumericFor(name, start, stop, step, body) => {
                let start = num_or_err(self.eval_expr_single(start)?, "for start")?;
                let stop = num_or_err(self.eval_expr_single(stop)?, "for stop")?;
                let step = match step {
                    Some(e) => num_or_err(self.eval_expr_single(e)?, "for step")?,
                    None => 1.0,
                };
                let mut i = start;
                loop {
                    if step > 0.0 && i > stop { break; }
                    if step < 0.0 && i < stop { break; }
                    if step == 0.0 { return Err(RtError::msg("'for' step is zero")); }
                    self.push_block_with_local(name, Value::Number(i));
                    let r = self.execute_block(body);
                    self.pop_block_local();
                    match r? {
                        Flow::Normal => {}
                        Flow::Break => break,
                        other => return Ok(other),
                    }
                    i += step;
                }
                Ok(Flow::Normal)
            }
            Stat::GenericFor(names, exprs, body) => {
                let vals = self.eval_exp_list(exprs, 3)?;
                let iter = vals.get(0).cloned().unwrap_or(Value::Nil);
                let state = vals.get(1).cloned().unwrap_or(Value::Nil);
                let mut control = vals.get(2).cloned().unwrap_or(Value::Nil);
                loop {
                    let result = self.call_value(&iter, vec![state.clone(), control.clone()])?;
                    let first = result.get(0).cloned().unwrap_or(Value::Nil);
                    if matches!(first, Value::Nil) { return Ok(Flow::Normal); }
                    control = first.clone();

                    // Push frame-locals for each name; restore on iteration end
                    let saved = self.frames.last().map(|f| f.locals.len()).unwrap_or(0);
                    for (i, n) in names.iter().enumerate() {
                        let v = result.get(i).cloned().unwrap_or(Value::Nil);
                        self.declare_local(Rc::clone(n), v);
                    }
                    let r = self.execute_block(body);
                    if let Some(frame) = self.frames.last_mut() {
                        frame.locals.truncate(saved);
                    }
                    match r? {
                        Flow::Normal => {}
                        Flow::Break => return Ok(Flow::Normal),
                        other => return Ok(other),
                    }
                }
            }
            Stat::LocalFunction(name, body) => {
                // Declare local first so the function can refer to itself recursively
                self.declare_local(Rc::clone(name), Value::Nil);
                let closure = self.make_closure(body)?;
                let slot = self.find_slot(name).expect("just declared");
                *slot.borrow_mut() = Value::Function(Function::Lua(closure));
                Ok(Flow::Normal)
            }
            Stat::Return(exprs) => {
                let values = self.eval_exp_list_multi(exprs)?;
                Ok(Flow::Return(values))
            }
            Stat::Break => Ok(Flow::Break),
            Stat::Goto(name) => Ok(Flow::Goto(Rc::clone(name))),
            Stat::Label(_) => Ok(Flow::Normal),
        }
    }

    fn push_block_with_local(&mut self, name: &Rc<str>, val: Value) {
        let slot = Rc::new(RefCell::new(val));
        let frame = self.frames.last_mut().expect("no frame");
        frame.locals.push((Rc::clone(name), slot));
    }
    fn pop_block_local(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.locals.pop();
        }
    }

    fn assign_to(&mut self, target: &Expr, val: Value) -> Result<(), RtError> {
        match target {
            Expr::Name(n) => {
                self.assign_name(n, val);
                Ok(())
            }
            Expr::Index(obj, key) => {
                let o = self.eval_expr_single(obj)?;
                let k = self.eval_expr_single(key)?;
                self.table_set(&o, k, val)
            }
            Expr::Field(obj, name) => {
                let o = self.eval_expr_single(obj)?;
                let k = Value::Str(Rc::from(name.as_bytes()));
                self.table_set(&o, k, val)
            }
            _ => Err(RtError::msg("invalid assignment target")),
        }
    }

    pub fn table_set(&mut self, t: &Value, k: Value, v: Value) -> Result<(), RtError> {
        match t {
            Value::Table(tbl) => {
                tbl.borrow_mut().set(k, v);
                Ok(())
            }
            _ => Err(RtError::msg(format!(
                "attempt to index a {} value",
                t.type_name()
            ))),
        }
    }

    pub fn table_get(&mut self, t: &Value, k: &Value) -> Result<Value, RtError> {
        match t {
            Value::Table(tbl) => {
                let v = tbl.borrow().get(k);
                if !matches!(v, Value::Nil) {
                    return Ok(v);
                }
                let mt = tbl.borrow().metatable.clone();
                if let Some(mt) = mt {
                    let idx_key = Value::Str(Rc::from(b"__index".as_slice()));
                    let idx = mt.borrow().get(&idx_key);
                    match idx {
                        Value::Table(_) => return self.table_get(&idx, k),
                        Value::Function(_) => {
                            return self.call_value(&idx, vec![t.clone(), k.clone()])
                                .map(|mut r| r.drain(..).next().unwrap_or(Value::Nil));
                        }
                        _ => {}
                    }
                }
                Ok(Value::Nil)
            }
            Value::Str(s) => {
                // Support s[i] character indexing via PICO-8 convention
                if let Value::Number(n) = k {
                    let idx = *n as i64;
                    if idx >= 1 && idx <= s.len() as i64 {
                        return Ok(Value::Str(Rc::from(&s[(idx - 1) as usize..idx as usize])));
                    }
                    return Ok(Value::Nil);
                }
                // String library lookup via globals.string
                let strlib_key = Value::Str(Rc::from(b"string".as_slice()));
                let strlib = self.globals.borrow().get(&strlib_key);
                if let Value::Table(_) = &strlib {
                    return self.table_get(&strlib, k);
                }
                Ok(Value::Nil)
            }
            _ => Err(RtError::msg(format!(
                "attempt to index a {} value",
                t.type_name()
            ))),
        }
    }

    pub fn eval_expr_single(&mut self, e: &Expr) -> Result<Value, RtError> {
        let v = self.eval_expr_multi(e)?;
        Ok(v.into_iter().next().unwrap_or(Value::Nil))
    }

    pub fn eval_expr_multi(&mut self, e: &Expr) -> Result<Vec<Value>, RtError> {
        match e {
            Expr::Nil => Ok(vec![Value::Nil]),
            Expr::True => Ok(vec![Value::Bool(true)]),
            Expr::False => Ok(vec![Value::Bool(false)]),
            Expr::Number(n) => Ok(vec![Value::Number(*n)]),
            Expr::Str(b) => Ok(vec![Value::Str(Rc::clone(b))]),
            Expr::Vararg => {
                let frame = self.frames.last().ok_or_else(|| RtError::msg("no varargs"))?;
                Ok(frame.varargs.clone())
            }
            Expr::Name(n) => Ok(vec![self.resolve(n)]),
            Expr::Index(o, k) => {
                let ov = self.eval_expr_single(o)?;
                let kv = self.eval_expr_single(k)?;
                Ok(vec![self.table_get(&ov, &kv)?])
            }
            Expr::Field(o, name) => {
                let ov = self.eval_expr_single(o)?;
                let kv = Value::Str(Rc::from(name.as_bytes()));
                Ok(vec![self.table_get(&ov, &kv)?])
            }
            Expr::Call(_, _) | Expr::MethodCall(_, _, _) => self.eval_call_expr(e, true),
            Expr::Function(body) => {
                let c = self.make_closure(body)?;
                Ok(vec![Value::Function(Function::Lua(c))])
            }
            Expr::BinOp(op, a, b) => Ok(vec![self.eval_binop(*op, a, b)?]),
            Expr::UnOp(op, a) => Ok(vec![self.eval_unop(*op, a)?]),
            Expr::Table(fields) => Ok(vec![self.eval_table(fields)?]),
        }
    }

    fn eval_call_expr(&mut self, e: &Expr, _multi: bool) -> Result<Vec<Value>, RtError> {
        match e {
            Expr::Call(fexpr, args) => {
                let f = self.eval_expr_single(fexpr)?;
                let argvals = self.eval_exp_list_multi(args)?;
                self.call_value(&f, argvals)
            }
            Expr::MethodCall(obj, name, args) => {
                let o = self.eval_expr_single(obj)?;
                let f = self.table_get(&o, &Value::Str(Rc::from(name.as_bytes())))?;
                let mut argvals = vec![o];
                argvals.extend(self.eval_exp_list_multi(args)?);
                self.call_value(&f, argvals)
            }
            _ => self.eval_expr_multi(e),
        }
    }

    /// Evaluate an expression list keeping multi-returns expanded only on the
    /// last expression (Lua semantics).
    fn eval_exp_list_multi(&mut self, exprs: &[Expr]) -> Result<Vec<Value>, RtError> {
        let mut out = Vec::new();
        for (i, e) in exprs.iter().enumerate() {
            if i + 1 == exprs.len() {
                out.extend(self.eval_expr_multi(e)?);
            } else {
                out.push(self.eval_expr_single(e)?);
            }
        }
        Ok(out)
    }

    fn eval_exp_list(&mut self, exprs: &[Expr], expected: usize) -> Result<Vec<Value>, RtError> {
        let mut v = self.eval_exp_list_multi(exprs)?;
        v.resize(expected.max(v.len()), Value::Nil);
        Ok(v)
    }

    fn eval_table(&mut self, fields: &[TableField]) -> Result<Value, RtError> {
        let t = Rc::new(RefCell::new(TableInner::new()));
        let mut auto_idx: i64 = 1;
        for (i, f) in fields.iter().enumerate() {
            match f {
                TableField::Array(e) => {
                    if i + 1 == fields.len() {
                        // Last: expand multi-values
                        let vs = self.eval_expr_multi(e)?;
                        for v in vs {
                            t.borrow_mut().set(Value::Number(auto_idx as f64), v);
                            auto_idx += 1;
                        }
                    } else {
                        let v = self.eval_expr_single(e)?;
                        t.borrow_mut().set(Value::Number(auto_idx as f64), v);
                        auto_idx += 1;
                    }
                }
                TableField::KV(k, v) => {
                    let kk = self.eval_expr_single(k)?;
                    let vv = self.eval_expr_single(v)?;
                    t.borrow_mut().set(kk, vv);
                }
            }
        }
        Ok(Value::Table(t))
    }

    fn eval_binop(&mut self, op: BinOp, a: &Expr, b: &Expr) -> Result<Value, RtError> {
        match op {
            BinOp::And => {
                let va = self.eval_expr_single(a)?;
                if !va.truthy() { return Ok(va); }
                self.eval_expr_single(b)
            }
            BinOp::Or => {
                let va = self.eval_expr_single(a)?;
                if va.truthy() { return Ok(va); }
                self.eval_expr_single(b)
            }
            _ => {
                let va = self.eval_expr_single(a)?;
                let vb = self.eval_expr_single(b)?;
                self.apply_binop(op, va, vb)
            }
        }
    }

    fn apply_binop(&mut self, op: BinOp, a: Value, b: Value) -> Result<Value, RtError> {
        use BinOp::*;
        match op {
            Add | Sub | Mul | Div | Mod | Pow => {
                let x = a.as_number().ok_or_else(|| RtError::msg(format!("arith on {}", a.type_name())))?;
                let y = b.as_number().ok_or_else(|| RtError::msg(format!("arith on {}", b.type_name())))?;
                Ok(Value::Number(match op {
                    Add => x + y,
                    Sub => x - y,
                    Mul => x * y,
                    Div => x / y,
                    Mod => { let r = x - y * (x / y).floor(); r }
                    Pow => x.powf(y),
                    _ => unreachable!(),
                }))
            }
            Concat => {
                let sa = a.as_str().ok_or_else(|| RtError::msg(format!("concat {}", a.type_name())))?;
                let sb = b.as_str().ok_or_else(|| RtError::msg(format!("concat {}", b.type_name())))?;
                let mut out = Vec::with_capacity(sa.len() + sb.len());
                out.extend_from_slice(&sa);
                out.extend_from_slice(&sb);
                Ok(Value::Str(Rc::from(out.as_slice())))
            }
            Eq => Ok(Value::Bool(self.values_equal(&a, &b))),
            NotEq => Ok(Value::Bool(!self.values_equal(&a, &b))),
            Lt => self.compare(&a, &b, |x, y| x < y, |sa, sb| sa < sb),
            Le => self.compare(&a, &b, |x, y| x <= y, |sa, sb| sa <= sb),
            Gt => self.compare(&a, &b, |x, y| x > y, |sa, sb| sa > sb),
            Ge => self.compare(&a, &b, |x, y| x >= y, |sa, sb| sa >= sb),
            And | Or => unreachable!(),
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        a.raw_equal(b)
    }

    fn compare<F, G>(&self, a: &Value, b: &Value, fnum: F, fstr: G) -> Result<Value, RtError>
    where F: Fn(f64, f64) -> bool, G: Fn(&[u8], &[u8]) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Bool(fnum(*x, *y))),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Bool(fstr(x, y))),
            // Lua semantics: nil compared with anything via < <= > >= is an error,
            // but the cart may rely on missing fields defaulting to 0 in some places.
            // For now, raise — this surfaces missing-field bugs in the cart.
            _ => Err(RtError::msg(format!(
                "compare {} with {}", a.type_name(), b.type_name()
            ))),
        }
    }

    fn eval_unop(&mut self, op: UnOp, e: &Expr) -> Result<Value, RtError> {
        let v = self.eval_expr_single(e)?;
        match op {
            UnOp::Neg => {
                let n = v.as_number().ok_or_else(|| RtError::msg("negate non-number"))?;
                Ok(Value::Number(-n))
            }
            UnOp::Not => Ok(Value::Bool(!v.truthy())),
            UnOp::Len => match v {
                Value::Str(s) => Ok(Value::Number(s.len() as f64)),
                Value::Table(t) => Ok(Value::Number(t.borrow().raw_len() as f64)),
                _ => Err(RtError::msg(format!("# on {}", v.type_name()))),
            },
        }
    }

    fn make_closure(&mut self, body: &FuncBody) -> Result<Rc<LuaClosure>, RtError> {
        // Capture all currently-visible locals + upvalues as upvalues for the closure.
        // Simple approach: scan the function body for free names, capture matching slots.
        let mut upvalues: Vec<(Rc<str>, Rc<RefCell<Value>>)> = Vec::new();
        let frame = match self.frames.last() { Some(f) => f, None => return Ok(Rc::new(LuaClosure {
            body: Rc::new(body.clone()),
            upvalues: Vec::new(),
            upnames: Vec::new(),
        }))};
        // Capture a snapshot of all current locals + outer upvalues.
        for (n, slot) in &frame.locals {
            upvalues.push((Rc::clone(n), Rc::clone(slot)));
        }
        for (n, slot) in &frame.upvalues {
            // Avoid shadowing same name — keep innermost if duplicates
            if !upvalues.iter().any(|(x, _)| x == n) {
                upvalues.push((Rc::clone(n), Rc::clone(slot)));
            }
        }
        let upnames: Vec<Rc<str>> = upvalues.iter().map(|(n, _)| Rc::clone(n)).collect();
        let upslots: Vec<Rc<RefCell<Value>>> = upvalues.into_iter().map(|(_, s)| s).collect();
        Ok(Rc::new(LuaClosure {
            body: Rc::new(body.clone()),
            upvalues: upslots,
            upnames,
        }))
    }

    pub fn call_value(&mut self, f: &Value, args: Vec<Value>) -> Result<Vec<Value>, RtError> {
        match f {
            Value::Function(Function::Native(n)) => {
                (n.func)(self, args)
            }
            Value::Function(Function::Lua(c)) => {
                self.call_lua_closure(c, args)
            }
            Value::Table(t) => {
                // __call metamethod
                let mt = t.borrow().metatable.clone();
                if let Some(mt) = mt {
                    let f = mt.borrow().get(&Value::Str(Rc::from(b"__call".as_slice())));
                    if !matches!(f, Value::Nil) {
                        let mut a = vec![Value::Table(Rc::clone(t))];
                        a.extend(args);
                        return self.call_value(&f, a);
                    }
                }
                Err(RtError::msg("attempt to call a table value"))
            }
            v => Err(RtError::msg(format!("attempt to call a {} value", v.type_name()))),
        }
    }

    fn call_lua_closure(&mut self, c: &Rc<LuaClosure>, mut args: Vec<Value>) -> Result<Vec<Value>, RtError> {
        let body = Rc::clone(&c.body);
        let mut frame = Frame {
            locals: Vec::new(),
            varargs: Vec::new(),
            upvalues: c.upnames.iter().cloned().zip(c.upvalues.iter().cloned()).collect(),
        };
        // Bind params
        let n_params = body.params.len();
        for (i, p) in body.params.iter().enumerate() {
            let v = if i < args.len() { args[i].clone() } else { Value::Nil };
            frame.locals.push((Rc::clone(p), Rc::new(RefCell::new(v))));
        }
        if body.is_vararg {
            if args.len() > n_params {
                frame.varargs = args.drain(n_params..).collect();
            }
        }
        self.frames.push(frame);
        let result = self.execute_block(&body.body);
        self.frames.pop();
        match result {
            Ok(Flow::Normal) => Ok(Vec::new()),
            Ok(Flow::Return(vs)) => Ok(vs),
            Ok(Flow::Break) => Err(RtError::msg("'break' outside loop")),
            Ok(Flow::Goto(name)) => Err(RtError::msg(format!("no visible label '{}'", name))),
            Err(e) => Err(e),
        }
    }
}

fn num_or_err(v: Value, what: &str) -> Result<f64, RtError> {
    v.as_number().ok_or_else(|| RtError::msg(format!("{} must be a number", what)))
}
