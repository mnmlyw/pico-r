//! Shadow model of z8lua's table layout, used only to reproduce the order
//! `pairs()`/`next()` yields.
//!
//! pico-r stores tables in an insertion-ordered map, which is fine for lookup
//! but gives the wrong iteration order: real PICO-8 walks the ARRAY part
//! ascending and then the NODE (hash) part in raw slot order, so the order
//! depends on where each key hashes, on collision displacement, and on the
//! table's growth history. Sorting cannot reproduce it -- the same key set
//! inserted in two different orders genuinely iterates differently
//! (`100,200,300,400` -> `300,100,200,400`, but inserted in reverse ->
//! `300,400,200,100`).
//!
//! So this mirrors the real structure alongside the real storage: every write
//! is replayed into a miniature Lua 5.2 table whose only job is to answer
//! "what order?". It is stock `ltable.c` with exactly two measured deviations,
//! both marked DEVIATION below.
//!
//! Derived entirely from the binary (no z8lua source was available). The rules
//! were cross-checked by implementing them twice, independently, and testing
//! the second implementation against 793 freshly generated key sets it had
//! never seen -- including random fuzz over 40-120-operation histories, deep
//! collision chains with the chain head deleted, and dead keys exactly filling
//! the node array. Zero mismatches.
//!
//! Scope: NUMERIC keys only. A table containing any non-numeric key is not
//! modelled and the caller falls back to its previous ordering. That is not
//! laziness -- string-key order on real hardware is genuinely random per run
//! (z8lua seeds its string hash from the clock), so there is no stable fact to
//! reproduce. What is NOT established, and would be needed to do better for
//! mixed tables, is exactly how non-numeric keys contribute to the node sizing
//! that the numeric keys' positions then depend on.

const INT32_MIN: i32 = i32::MIN;
const MAXBITS: usize = 30;
const MAXASIZE: i64 = 1 << 30;

/// A number's 16.16 fixed-point bits. Two keys are the same key iff their raw
/// values match, so this is the only identity the layout needs.
pub fn raw_of(n: f64) -> Option<i32> {
    if !n.is_finite() {
        return None;
    }
    let scaled = (n * 65536.0).round();
    if scaled.abs() >= 9.0e18 {
        return None;
    }
    // WRAPS rather than saturating or rejecting. A 16.16 value is a 32-bit
    // register on the console, so a key past +-32767 comes back around --
    // 32768 and -32768 are the same key there. pico-r's own f64 does not
    // wrap (a separate divergence), but the layout must be computed on the
    // wrapped bits or such a key lands in the wrong slot.
    Some((scaled as i64 as u32) as i32)
}

fn is_int_key(raw: i32) -> bool {
    raw & 0xFFFF == 0
}

fn int_of(raw: i32) -> i64 {
    (raw >> 16) as i64
}

fn ceillog2(x: i64) -> usize {
    let mut n = 0usize;
    while (1i64 << n) < x {
        n += 1;
    }
    n
}

/// z8lua's number hash. Stock Lua 5.2 hashes the float's bit pattern; here the
/// number IS the 16.16 int32, and the fold is a plain negation with INT_MIN
/// (key -32768) collapsing to 0 rather than staying negative.
fn hash_index(raw: i32) -> u32 {
    if raw < 0 {
        if raw == INT32_MIN {
            0
        } else {
            (-(raw as i64)) as u32
        }
    } else {
        raw as u32
    }
}

#[derive(Clone, Default)]
struct Slot {
    key: Option<i32>,
    /// Only liveness matters for ordering, never the value itself. A slot with
    /// a key but no value is a "dead key": invisible to pairs(), still found by
    /// lookup, and still blocking `getfreepos`.
    live: bool,
    next: Option<usize>,
}

#[derive(Clone, Default)]
pub struct Layout {
    /// 1-based; `arr[i]` is whether array slot i holds a live value.
    arr: Vec<bool>,
    node: Vec<Slot>,
    lastfree: usize,
    /// Set once a non-numeric key is stored. The model says nothing useful
    /// about those, so the whole table stops being modelled.
    pub unmodelled: bool,
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    fn sizearray(&self) -> i64 {
        self.arr.len() as i64
    }

    fn sizenode(&self) -> usize {
        self.node.len()
    }

    /// DEVIATION 1: modulo `sizenode-1`, NOT the `& (sizenode-1)` bitmask Lua
    /// uses for string keys. Measured directly by building collision-free
    /// tables with one key per residue class and reading the slot order back
    /// out of pairs(); the bitmask form disagrees with the console immediately.
    fn mainposition(&self, raw: i32) -> usize {
        let m = ((self.sizenode() as i64 - 1) | 1) as u32;
        (hash_index(raw) % m) as usize
    }

    fn in_array(&self, raw: i32) -> bool {
        is_int_key(raw) && (1..=self.sizearray()).contains(&int_of(raw))
    }

    fn find_node(&self, raw: i32) -> Option<usize> {
        if self.sizenode() == 0 {
            return None;
        }
        let mut n = Some(self.mainposition(raw));
        while let Some(i) = n {
            if self.node[i].key == Some(raw) {
                return Some(i);
            }
            n = self.node[i].next;
        }
        None
    }

    /// Record a write. `live` is false for `t[k] = nil`.
    ///
    /// Note a nil store to an ABSENT key is not a no-op: Lua 5.2 still creates
    /// the entry, which can trigger a rehash and can even grow the array part.
    pub fn set(&mut self, raw: i32, live: bool) {
        if self.in_array(raw) {
            let i = int_of(raw) as usize;
            self.arr[i - 1] = live;
            return;
        }
        if let Some(n) = self.find_node(raw) {
            self.node[n].live = live;
            return;
        }
        self.newkey(raw, live);
    }

    /// Scans DOWNWARD from the cursor, and tests the KEY, so a dead key still
    /// occupies its slot.
    fn getfreepos(&mut self) -> Option<usize> {
        while self.lastfree > 0 {
            self.lastfree -= 1;
            if self.node[self.lastfree].key.is_none() {
                return Some(self.lastfree);
            }
        }
        None
    }

    fn newkey(&mut self, raw: i32, live: bool) {
        if self.sizenode() == 0 {
            self.rehash(Some(raw));
            self.set(raw, live);
            return;
        }
        let mut mp = self.mainposition(raw);
        if self.node[mp].live {
            let Some(f) = self.getfreepos() else {
                self.rehash(Some(raw));
                self.set(raw, live);
                return;
            };
            let othern = self.mainposition(self.node[mp].key.unwrap());
            if othern != mp {
                // Brent's variation: the occupant is not in its own main
                // position, so it gets relocated and the newcomer takes the
                // slot it actually hashes to.
                let mut prev = othern;
                while self.node[prev].next != Some(mp) {
                    prev = self.node[prev].next.unwrap();
                }
                self.node[prev].next = Some(f);
                self.node[f] = self.node[mp].clone();
                self.node[mp] = Slot::default();
            } else {
                self.node[f].next = self.node[mp].next;
                self.node[mp].next = Some(f);
                mp = f;
            }
        }
        self.node[mp].key = Some(raw);
        self.node[mp].live = live;
    }

    fn countint(raw: i32, nums: &mut [i64]) -> i64 {
        if is_int_key(raw) && int_of(raw) > 0 && int_of(raw) <= MAXASIZE {
            nums[ceillog2(int_of(raw))] += 1;
            1
        } else {
            0
        }
    }

    // Deliberately mirrors ltable.c's index arithmetic rather than idiomatic
    // Rust iteration: `i` carries across iterations and `lim` can cut a bucket
    // short, so the loop variable is not a plain index into `nums`.
    #[allow(clippy::needless_range_loop)]
    fn numusearray(&self, nums: &mut [i64]) -> i64 {
        let mut ause = 0i64;
        let mut i = 1i64;
        let mut ttlg = 1i64;
        for lg in 0..=MAXBITS {
            let mut lc = 0i64;
            let mut lim = ttlg;
            if lim > self.sizearray() {
                lim = self.sizearray();
                if i > lim {
                    break;
                }
            }
            while i <= lim {
                if self.arr[(i - 1) as usize] {
                    lc += 1;
                }
                i += 1;
            }
            nums[lg] += lc;
            ause += lc;
            ttlg *= 2;
        }
        ause
    }

    fn numusehash(&self, nums: &mut [i64]) -> (i64, i64) {
        let mut totaluse = 0i64;
        let mut ause = 0i64;
        for s in &self.node {
            if s.live {
                ause += Self::countint(s.key.unwrap(), nums);
                totaluse += 1;
            }
        }
        (totaluse, ause)
    }

    fn computesizes(nums: &[i64], pna: i64) -> (i64, i64) {
        let (mut a, mut na, mut n) = (0i64, 0i64, 0i64);
        let mut i = 0usize;
        let mut twotoi = 1i64;
        while twotoi / 2 < pna {
            if nums[i] > 0 {
                a += nums[i];
                if a > twotoi / 2 {
                    n = twotoi;
                    na = a;
                }
            }
            if a == pna {
                break;
            }
            i += 1;
            twotoi *= 2;
        }
        (na, n)
    }

    fn rehash(&mut self, extra: Option<i32>) {
        let mut nums = vec![0i64; MAXBITS + 2];
        let mut nasize = self.numusearray(&mut nums);
        let mut totaluse = nasize;
        let (tu, au) = self.numusehash(&mut nums);
        totaluse += tu;
        nasize += au;
        if let Some(e) = extra {
            nasize += Self::countint(e, &mut nums);
            totaluse += 1;
        }
        let (na, optsize) = Self::computesizes(&nums, nasize);
        self.resize(optsize, totaluse - na);
    }

    fn resize(&mut self, nasize: i64, nhsize: i64) {
        // DEVIATION 2: the array part never shrinks. Stock Lua reallocates it
        // smaller and moves the displaced keys into the node part; here a
        // once-grown array stays grown, which changes iteration order for any
        // table that is filled and then emptied.
        let nasize = nasize.max(self.sizearray());
        let old = std::mem::take(&mut self.node);
        if nasize > self.sizearray() {
            self.arr.resize(nasize as usize, false);
        }
        if nhsize == 0 {
            self.node = Vec::new();
            self.lastfree = 0;
        } else {
            let sz = 1usize << ceillog2(nhsize);
            self.node = vec![Slot::default(); sz];
            self.lastfree = sz;
        }
        // Reinserted from the TOP down, which is what makes the growth history
        // visible in the final order.
        for s in old.iter().rev() {
            if s.live {
                self.set(s.key.unwrap(), true);
            }
        }
    }

    /// Keys in `pairs()` order: array part ascending, then node slots in raw
    /// index order, skipping anything not live.
    pub fn order(&self) -> Vec<i32> {
        let mut out = Vec::new();
        for (i, live) in self.arr.iter().enumerate() {
            if *live {
                out.push(((i as i32) + 1) << 16);
            }
        }
        for s in &self.node {
            if s.live {
                out.push(s.key.unwrap());
            }
        }
        out
    }
}
