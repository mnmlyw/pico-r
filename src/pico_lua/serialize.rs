// Serialize/deserialize Lua values for save state.
//
// Format (tagged, little-endian):
//   0x00 = nil
//   0x01 = bool false
//   0x02 = bool true
//   0x03 = number (8 bytes f64)
//   0x04 = string (u32 len + bytes)
//   0x05 = table ref (u32 id) — for already-seen tables (cycle/sharing)
//   0x06 = table def (u32 id, then key/value pairs, terminated by 0x00 key tag)
//
// Only the globals table is serialized; functions and unreachable values are
// skipped. The cart source must already be loaded on the receiver so that
// function definitions are in place; deserialization just reinstates state.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::value::*;

// ---- Save ----

pub fn save_globals(globals: &Rc<Table>) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4096);
    let mut seen: HashMap<usize, u32> = HashMap::new();
    let mut next_id: u32 = 1;
    let g = globals.borrow();
    // Count serializable pairs
    let mut count: u32 = 0;
    for (k, v) in g.map.iter() {
        if !is_serializable(v) {
            continue;
        }
        if !is_serializable_key(k) {
            continue;
        }
        count += 1;
    }
    // Header: u32 magic, u32 count
    buf.extend_from_slice(b"PICR");
    buf.extend_from_slice(&count.to_le_bytes());
    for (k, v) in g.map.iter() {
        if !is_serializable(v) {
            continue;
        }
        if !is_serializable_key(k) {
            continue;
        }
        write_key(&mut buf, k);
        write_value(&mut buf, v, &mut seen, &mut next_id);
    }
    buf
}

fn is_serializable(v: &Value) -> bool {
    !matches!(v, Value::Function(_))
}

fn is_serializable_key(k: &Key) -> bool {
    !matches!(k, Key::Function(_) | Key::Table(_))
}

fn write_key(buf: &mut Vec<u8>, k: &Key) {
    match k {
        Key::Bool(b) => {
            buf.push(if *b { 0x02 } else { 0x01 });
        }
        Key::Int(i) => {
            buf.push(0x03);
            buf.extend_from_slice(&(*i as f64).to_le_bytes());
        }
        Key::Float(bits) => {
            buf.push(0x03);
            buf.extend_from_slice(&bits.to_le_bytes());
        }
        Key::Str(s) => {
            buf.push(0x04);
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s);
        }
        Key::Table(_) | Key::Function(_) => {} // unreachable due to filter
    }
}

fn write_value(buf: &mut Vec<u8>, v: &Value, seen: &mut HashMap<usize, u32>, next_id: &mut u32) {
    match v {
        Value::Nil => buf.push(0x00),
        Value::Bool(b) => buf.push(if *b { 0x02 } else { 0x01 }),
        Value::Number(n) => {
            buf.push(0x03);
            buf.extend_from_slice(&n.to_le_bytes());
        }
        Value::Str(s) => {
            buf.push(0x04);
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s);
        }
        Value::Function(_) => buf.push(0x00), // skip funcs as nil
        Value::Table(t) => {
            let ptr = Rc::as_ptr(t) as usize;
            if let Some(&id) = seen.get(&ptr) {
                buf.push(0x05);
                buf.extend_from_slice(&id.to_le_bytes());
                return;
            }
            let id = *next_id;
            *next_id += 1;
            seen.insert(ptr, id);
            buf.push(0x06);
            buf.extend_from_slice(&id.to_le_bytes());
            for (k, vv) in t.borrow().map.iter() {
                if !is_serializable(vv) {
                    continue;
                }
                if !is_serializable_key(k) {
                    continue;
                }
                write_key(buf, k);
                write_value(buf, vv, seen, next_id);
            }
            buf.push(0x00); // end-of-table marker (nil key)
        }
    }
}

// ---- Load ----

pub struct Loader<'a> {
    pub data: &'a [u8],
    pub pos: usize,
    pub tables: HashMap<u32, Rc<Table>>,
}

impl<'a> Loader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            tables: HashMap::new(),
        }
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() {
            return None;
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Some(b)
    }
    pub fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 > self.data.len() {
            return None;
        }
        let v = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Some(v)
    }
    pub fn read_f64(&mut self) -> Option<f64> {
        if self.pos + 8 > self.data.len() {
            return None;
        }
        let mut b = [0u8; 8];
        b.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Some(f64::from_le_bytes(b))
    }

    pub fn read_value(&mut self) -> Option<Value> {
        let tag = self.read_u8()?;
        match tag {
            0x00 => Some(Value::Nil),
            0x01 => Some(Value::Bool(false)),
            0x02 => Some(Value::Bool(true)),
            0x03 => Some(Value::Number(self.read_f64()?)),
            0x04 => {
                let len = self.read_u32()? as usize;
                if self.pos + len > self.data.len() {
                    return None;
                }
                let s = Rc::from(&self.data[self.pos..self.pos + len]);
                self.pos += len;
                Some(Value::Str(s))
            }
            0x05 => {
                let id = self.read_u32()?;
                self.tables.get(&id).map(|t| Value::Table(Rc::clone(t)))
            }
            0x06 => {
                let id = self.read_u32()?;
                let t = Rc::new(RefCell::new(TableInner::new()));
                self.tables.insert(id, Rc::clone(&t));
                loop {
                    let k = self.read_key()?;
                    if k.is_none() {
                        break;
                    }
                    let v = self.read_value()?;
                    t.borrow_mut().set(k.unwrap(), v);
                }
                Some(Value::Table(t))
            }
            _ => None,
        }
    }

    /// Returns Ok(None) for the end-of-table sentinel (0x00).
    pub fn read_key(&mut self) -> Option<Option<Value>> {
        let tag = self.read_u8()?;
        match tag {
            0x00 => Some(None), // end-of-table
            0x01 => Some(Some(Value::Bool(false))),
            0x02 => Some(Some(Value::Bool(true))),
            0x03 => Some(Some(Value::Number(self.read_f64()?))),
            0x04 => {
                let len = self.read_u32()? as usize;
                if self.pos + len > self.data.len() {
                    return None;
                }
                let s = Rc::from(&self.data[self.pos..self.pos + len]);
                self.pos += len;
                Some(Some(Value::Str(s)))
            }
            _ => None,
        }
    }
}

pub fn load_globals(globals: &Rc<Table>, data: &[u8]) -> Result<(), String> {
    if data.len() < 8 || &data[0..4] != b"PICR" {
        return Err("bad save magic".into());
    }
    // Wipe non-function globals so previous-frame state doesn't leak.
    let to_remove: Vec<Key> = {
        let g = globals.borrow();
        g.map
            .iter()
            .filter(|(_, v)| !matches!(v, Value::Function(_)))
            .map(|(k, _)| k.clone())
            .collect()
    };
    {
        let mut g = globals.borrow_mut();
        for k in to_remove {
            g.map.remove(&k);
        }
    }
    let mut ld = Loader::new(data);
    ld.pos = 4;
    let count = ld.read_u32().ok_or("truncated header")?;
    for _ in 0..count {
        let k = ld
            .read_key()
            .ok_or("truncated key")?
            .ok_or("nil key in globals")?;
        let v = ld.read_value().ok_or("truncated value")?;
        let existing = globals.borrow().get(&k);
        if !matches!(existing, Value::Function(_)) {
            globals.borrow_mut().set(k, v);
        }
    }
    Ok(())
}
