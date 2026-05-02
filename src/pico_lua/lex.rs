// Lua 5.2 tokenizer. Operates on bytes (no UTF-8 validation needed —
// Lua strings are byte strings). PICO-8 source has already been
// run through the preprocessor, so this only sees standard Lua 5.2 syntax.

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    // Literals
    Number(f64),
    Str(Rc<[u8]>),
    Name(Rc<str>),
    // Keywords
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    Goto,
    If,
    In,
    Local,
    Nil,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,
    // Punctuation / operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Hash,
    Eq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
    Assign,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    DoubleColon,
    Semi,
    Colon,
    Comma,
    Dot,
    Concat,
    Vararg,
    // End of file
    Eof,
}

pub struct Token {
    pub tok: Tok,
    pub line: u32,
}

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: u32,
}

#[derive(Debug)]
pub struct LexError {
    pub msg: String,
    pub line: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Self {
            src,
            pos: 0,
            line: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut out = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.pos >= self.src.len() {
                out.push(Token {
                    tok: Tok::Eof,
                    line: self.line,
                });
                return Ok(out);
            }
            let line = self.line;
            let tok = self.next_tok()?;
            out.push(Token { tok, line });
        }
    }

    fn skip_trivia(&mut self) -> Result<(), LexError> {
        loop {
            while self.pos < self.src.len() {
                match self.src[self.pos] {
                    b' ' | b'\t' | b'\r' => self.pos += 1,
                    b'\n' => {
                        self.pos += 1;
                        self.line += 1;
                    }
                    _ => break,
                }
            }
            // Comments
            if self.pos + 1 < self.src.len()
                && self.src[self.pos] == b'-'
                && self.src[self.pos + 1] == b'-'
            {
                self.pos += 2;
                // Long comment? --[[ or --[=[
                if self.pos < self.src.len() && self.src[self.pos] == b'[' {
                    if let Some(level) = self.try_long_open() {
                        self.skip_long_bracket(level)?;
                        continue;
                    }
                }
                // Short comment to end of line
                while self.pos < self.src.len() && self.src[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
        Ok(())
    }

    fn try_long_open(&mut self) -> Option<usize> {
        // Caller has confirmed self.src[self.pos] == b'[' (or about to).
        let start = self.pos;
        if self.src[start] != b'[' {
            return None;
        }
        let mut p = start + 1;
        let mut level = 0;
        while p < self.src.len() && self.src[p] == b'=' {
            level += 1;
            p += 1;
        }
        if p < self.src.len() && self.src[p] == b'[' {
            self.pos = p + 1;
            Some(level)
        } else {
            None
        }
    }

    fn skip_long_bracket(&mut self, level: usize) -> Result<(), LexError> {
        // Find ]=*]
        loop {
            if self.pos >= self.src.len() {
                return Err(self.err("unterminated long bracket"));
            }
            if self.src[self.pos] == b'\n' {
                self.line += 1;
            }
            if self.src[self.pos] == b']' {
                let mut p = self.pos + 1;
                let mut count = 0;
                while p < self.src.len() && self.src[p] == b'=' && count < level {
                    count += 1;
                    p += 1;
                }
                if count == level && p < self.src.len() && self.src[p] == b']' {
                    self.pos = p + 1;
                    return Ok(());
                }
            }
            self.pos += 1;
        }
    }

    fn read_long_string(&mut self, level: usize) -> Result<Vec<u8>, LexError> {
        let mut out = Vec::new();
        // Skip leading newline if present (Lua spec)
        if self.pos < self.src.len() && self.src[self.pos] == b'\n' {
            self.pos += 1;
            self.line += 1;
        } else if self.pos + 1 < self.src.len()
            && self.src[self.pos] == b'\r'
            && self.src[self.pos + 1] == b'\n'
        {
            self.pos += 2;
            self.line += 1;
        }
        loop {
            if self.pos >= self.src.len() {
                return Err(self.err("unterminated long string"));
            }
            if self.src[self.pos] == b']' {
                let mut p = self.pos + 1;
                let mut count = 0;
                while p < self.src.len() && self.src[p] == b'=' && count < level {
                    count += 1;
                    p += 1;
                }
                if count == level && p < self.src.len() && self.src[p] == b']' {
                    self.pos = p + 1;
                    return Ok(out);
                }
            }
            if self.src[self.pos] == b'\n' {
                self.line += 1;
            }
            out.push(self.src[self.pos]);
            self.pos += 1;
        }
    }

    fn next_tok(&mut self) -> Result<Tok, LexError> {
        let c = self.src[self.pos];
        match c {
            b'+' => {
                self.pos += 1;
                Ok(Tok::Plus)
            }
            b'-' => {
                self.pos += 1;
                Ok(Tok::Minus)
            }
            b'*' => {
                self.pos += 1;
                Ok(Tok::Star)
            }
            b'/' => {
                self.pos += 1;
                Ok(Tok::Slash)
            }
            b'%' => {
                self.pos += 1;
                Ok(Tok::Percent)
            }
            b'^' => {
                self.pos += 1;
                Ok(Tok::Caret)
            }
            b'#' => {
                self.pos += 1;
                Ok(Tok::Hash)
            }
            b'(' => {
                self.pos += 1;
                Ok(Tok::LParen)
            }
            b')' => {
                self.pos += 1;
                Ok(Tok::RParen)
            }
            b'{' => {
                self.pos += 1;
                Ok(Tok::LBrace)
            }
            b'}' => {
                self.pos += 1;
                Ok(Tok::RBrace)
            }
            b';' => {
                self.pos += 1;
                Ok(Tok::Semi)
            }
            b',' => {
                self.pos += 1;
                Ok(Tok::Comma)
            }
            b'[' => {
                // Long string?
                if let Some(level) = self.try_long_open() {
                    let bytes = self.read_long_string(level)?;
                    return Ok(Tok::Str(Rc::from(bytes.as_slice())));
                }
                self.pos += 1;
                Ok(Tok::LBracket)
            }
            b']' => {
                self.pos += 1;
                Ok(Tok::RBracket)
            }
            b'=' => {
                if self.peek_eq(1, b'=') {
                    self.pos += 2;
                    Ok(Tok::Eq)
                } else {
                    self.pos += 1;
                    Ok(Tok::Assign)
                }
            }
            b'~' => {
                if self.peek_eq(1, b'=') {
                    self.pos += 2;
                    Ok(Tok::NotEq)
                } else {
                    Err(self.err("expected '=' after '~'"))
                }
            }
            b'<' => {
                if self.peek_eq(1, b'=') {
                    self.pos += 2;
                    Ok(Tok::Le)
                } else {
                    self.pos += 1;
                    Ok(Tok::Lt)
                }
            }
            b'>' => {
                if self.peek_eq(1, b'=') {
                    self.pos += 2;
                    Ok(Tok::Ge)
                } else {
                    self.pos += 1;
                    Ok(Tok::Gt)
                }
            }
            b':' => {
                if self.peek_eq(1, b':') {
                    self.pos += 2;
                    Ok(Tok::DoubleColon)
                } else {
                    self.pos += 1;
                    Ok(Tok::Colon)
                }
            }
            b'.' => {
                if self.peek_eq(1, b'.') {
                    if self.peek_eq(2, b'.') {
                        self.pos += 3;
                        Ok(Tok::Vararg)
                    } else {
                        self.pos += 2;
                        Ok(Tok::Concat)
                    }
                } else if self.pos + 1 < self.src.len() && self.src[self.pos + 1].is_ascii_digit() {
                    self.read_number()
                } else {
                    self.pos += 1;
                    Ok(Tok::Dot)
                }
            }
            b'"' | b'\'' => self.read_short_string(c),
            b'0'..=b'9' => self.read_number(),
            b'_' | b'a'..=b'z' | b'A'..=b'Z' => self.read_identifier(),
            _ => Err(self.err_byte("unexpected character", c)),
        }
    }

    fn peek_eq(&self, off: usize, b: u8) -> bool {
        self.pos + off < self.src.len() && self.src[self.pos + off] == b
    }

    fn read_short_string(&mut self, quote: u8) -> Result<Tok, LexError> {
        self.pos += 1;
        let mut out: Vec<u8> = Vec::new();
        while self.pos < self.src.len() {
            let c = self.src[self.pos];
            if c == quote {
                self.pos += 1;
                return Ok(Tok::Str(Rc::from(out.as_slice())));
            }
            if c == b'\n' {
                return Err(self.err("unterminated string"));
            }
            if c == b'\\' {
                self.pos += 1;
                if self.pos >= self.src.len() {
                    return Err(self.err("unterminated escape"));
                }
                let esc = self.src[self.pos];
                match esc {
                    b'a' => {
                        out.push(0x07);
                        self.pos += 1;
                    }
                    b'b' => {
                        out.push(0x08);
                        self.pos += 1;
                    }
                    b'f' => {
                        out.push(0x0C);
                        self.pos += 1;
                    }
                    b'n' => {
                        out.push(b'\n');
                        self.pos += 1;
                    }
                    b'r' => {
                        out.push(b'\r');
                        self.pos += 1;
                    }
                    b't' => {
                        out.push(b'\t');
                        self.pos += 1;
                    }
                    b'v' => {
                        out.push(0x0B);
                        self.pos += 1;
                    }
                    b'\\' => {
                        out.push(b'\\');
                        self.pos += 1;
                    }
                    b'"' => {
                        out.push(b'"');
                        self.pos += 1;
                    }
                    b'\'' => {
                        out.push(b'\'');
                        self.pos += 1;
                    }
                    b'\n' => {
                        out.push(b'\n');
                        self.line += 1;
                        self.pos += 1;
                    }
                    b'\r' => {
                        out.push(b'\n');
                        self.line += 1;
                        self.pos += 1;
                        if self.pos < self.src.len() && self.src[self.pos] == b'\n' {
                            self.pos += 1;
                        }
                    }
                    b'x' => {
                        self.pos += 1;
                        let mut v: u32 = 0;
                        let mut n = 0;
                        while n < 2 && self.pos < self.src.len() && is_hex(self.src[self.pos]) {
                            v = v * 16 + hex_val(self.src[self.pos]) as u32;
                            self.pos += 1;
                            n += 1;
                        }
                        if n == 0 {
                            return Err(self.err("malformed \\x escape"));
                        }
                        out.push(v as u8);
                    }
                    b'z' => {
                        self.pos += 1;
                        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace()
                        {
                            if self.src[self.pos] == b'\n' {
                                self.line += 1;
                            }
                            self.pos += 1;
                        }
                    }
                    b'0'..=b'9' => {
                        let mut v: u32 = 0;
                        let mut n = 0;
                        while n < 3
                            && self.pos < self.src.len()
                            && self.src[self.pos].is_ascii_digit()
                        {
                            v = v * 10 + (self.src[self.pos] - b'0') as u32;
                            self.pos += 1;
                            n += 1;
                        }
                        if v > 255 {
                            return Err(self.err("decimal escape > 255"));
                        }
                        out.push(v as u8);
                    }
                    _ => return Err(self.err_byte("invalid escape", esc)),
                }
            } else {
                out.push(c);
                self.pos += 1;
            }
        }
        Err(self.err("unterminated string"))
    }

    fn read_number(&mut self) -> Result<Tok, LexError> {
        let start = self.pos;
        let mut is_hex = false;
        if self.src[self.pos] == b'0'
            && self.pos + 1 < self.src.len()
            && (self.src[self.pos + 1] == b'x' || self.src[self.pos + 1] == b'X')
        {
            is_hex = true;
            self.pos += 2;
            while self.pos < self.src.len() && is_hex_byte(self.src[self.pos]) {
                self.pos += 1;
            }
            if self.pos < self.src.len() && self.src[self.pos] == b'.' {
                self.pos += 1;
                while self.pos < self.src.len() && is_hex_byte(self.src[self.pos]) {
                    self.pos += 1;
                }
            }
            // hex exponent (p)
            if self.pos < self.src.len()
                && (self.src[self.pos] == b'p' || self.src[self.pos] == b'P')
            {
                self.pos += 1;
                if self.pos < self.src.len()
                    && (self.src[self.pos] == b'+' || self.src[self.pos] == b'-')
                {
                    self.pos += 1;
                }
                while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }
        } else {
            while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            if self.pos < self.src.len() && self.src[self.pos] == b'.' {
                self.pos += 1;
                while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }
            if self.pos < self.src.len()
                && (self.src[self.pos] == b'e' || self.src[self.pos] == b'E')
            {
                self.pos += 1;
                if self.pos < self.src.len()
                    && (self.src[self.pos] == b'+' || self.src[self.pos] == b'-')
                {
                    self.pos += 1;
                }
                while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }
        }
        let s = unsafe { std::str::from_utf8_unchecked(&self.src[start..self.pos]) };
        let n = if is_hex {
            parse_hex_float(s).ok_or_else(|| self.err("bad hex number"))?
        } else {
            s.parse::<f64>().map_err(|_| self.err("bad number"))?
        };
        Ok(Tok::Number(n))
    }

    fn read_identifier(&mut self) -> Result<Tok, LexError> {
        let start = self.pos;
        while self.pos < self.src.len() {
            let c = self.src[self.pos];
            if c == b'_' || c.is_ascii_alphanumeric() {
                self.pos += 1;
            } else {
                break;
            }
        }
        let bytes = &self.src[start..self.pos];
        Ok(match bytes {
            b"and" => Tok::And,
            b"break" => Tok::Break,
            b"do" => Tok::Do,
            b"else" => Tok::Else,
            b"elseif" => Tok::Elseif,
            b"end" => Tok::End,
            b"false" => Tok::False,
            b"for" => Tok::For,
            b"function" => Tok::Function,
            b"goto" => Tok::Goto,
            b"if" => Tok::If,
            b"in" => Tok::In,
            b"local" => Tok::Local,
            b"nil" => Tok::Nil,
            b"not" => Tok::Not,
            b"or" => Tok::Or,
            b"repeat" => Tok::Repeat,
            b"return" => Tok::Return,
            b"then" => Tok::Then,
            b"true" => Tok::True,
            b"until" => Tok::Until,
            b"while" => Tok::While,
            _ => {
                let s = std::str::from_utf8(bytes).map_err(|_| self.err("non-utf8 identifier"))?;
                Tok::Name(Rc::from(s))
            }
        })
    }

    fn err(&self, msg: &str) -> LexError {
        LexError {
            msg: msg.into(),
            line: self.line,
        }
    }
    fn err_byte(&self, msg: &str, b: u8) -> LexError {
        LexError {
            msg: format!("{} (byte 0x{:02x})", msg, b),
            line: self.line,
        }
    }
}

fn is_hex(b: u8) -> bool {
    is_hex_byte(b)
}
fn is_hex_byte(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b)
}
fn hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        _ => b - b'A' + 10,
    }
}

fn parse_hex_float(s: &str) -> Option<f64> {
    // Parse "0xHHHH[.HHHH][pNN]"
    let bytes = s.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'0' || (bytes[1] != b'x' && bytes[1] != b'X') {
        return None;
    }
    let mut i = 2;
    let mut int_part: u64 = 0;
    while i < bytes.len() && is_hex_byte(bytes[i]) {
        int_part = int_part
            .checked_mul(16)?
            .checked_add(hex_val(bytes[i]) as u64)?;
        i += 1;
    }
    let mut frac: f64 = 0.0;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let mut scale = 1.0 / 16.0;
        while i < bytes.len() && is_hex_byte(bytes[i]) {
            frac += hex_val(bytes[i]) as f64 * scale;
            scale /= 16.0;
            i += 1;
        }
    }
    let mut exp: i32 = 0;
    if i < bytes.len() && (bytes[i] == b'p' || bytes[i] == b'P') {
        i += 1;
        let neg = if i < bytes.len() && bytes[i] == b'-' {
            i += 1;
            true
        } else if i < bytes.len() && bytes[i] == b'+' {
            i += 1;
            false
        } else {
            false
        };
        let mut e = 0i32;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            e = e * 10 + (bytes[i] - b'0') as i32;
            i += 1;
        }
        exp = if neg { -e } else { e };
    }
    if i != bytes.len() {
        return None;
    }
    let val = (int_part as f64 + frac) * 2f64.powi(exp);
    Some(val)
}
