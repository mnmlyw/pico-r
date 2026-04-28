// Lua 5.2 grammar parser. Produces AST. Reasonably faithful but not paranoid:
// the preprocessor only emits well-formed Lua, so bad-syntax error messages
// are basic.

use std::rc::Rc;

use super::ast::*;
use super::lex::{Tok, Token};

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub line: u32,
}

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self {
        Self { toks, pos: 0 }
    }

    pub fn parse_chunk(&mut self) -> Result<Block, ParseError> {
        let block = self.parse_block()?;
        if !self.is(&Tok::Eof) {
            return Err(self.err("unexpected token after chunk"));
        }
        Ok(block)
    }

    fn peek(&self) -> &Tok { &self.toks[self.pos].tok }
    fn line(&self) -> u32 { self.toks[self.pos].line }
    fn is(&self, t: &Tok) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(t)
    }
    fn check(&mut self, t: &Tok) -> bool { self.is(t) }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.is(t) { self.pos += 1; true } else { false }
    }
    fn expect(&mut self, t: &Tok, name: &str) -> Result<(), ParseError> {
        if self.eat(t) { Ok(()) } else { Err(self.err(&format!("expected {}", name))) }
    }
    fn advance(&mut self) -> Tok {
        let t = self.toks[self.pos].tok.clone();
        self.pos += 1;
        t
    }

    fn err(&self, msg: &str) -> ParseError {
        ParseError { msg: msg.into(), line: self.line() }
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let line = self.line();
        let mut stats = Vec::new();
        let mut stat_lines = Vec::new();
        loop {
            while self.eat(&Tok::Semi) {}
            if self.is_block_terminator() { break; }
            let stat_line = self.line();
            let s = self.parse_statement()?;
            let is_return = matches!(s, Stat::Return(_));
            stats.push(s);
            stat_lines.push(stat_line);
            if is_return {
                while self.eat(&Tok::Semi) {}
                break;
            }
        }
        Ok(Block { stats, stat_lines, line })
    }

    fn is_block_terminator(&self) -> bool {
        matches!(
            self.peek(),
            Tok::End | Tok::Else | Tok::Elseif | Tok::Until | Tok::Eof
        )
    }

    fn parse_statement(&mut self) -> Result<Stat, ParseError> {
        match self.peek() {
            Tok::If => self.parse_if(),
            Tok::While => self.parse_while(),
            Tok::Do => { self.pos += 1; let b = self.parse_block()?; self.expect(&Tok::End, "'end'")?; Ok(Stat::Do(b)) }
            Tok::For => self.parse_for(),
            Tok::Repeat => self.parse_repeat(),
            Tok::Function => self.parse_function_stat(),
            Tok::Local => self.parse_local(),
            Tok::Return => { self.pos += 1; self.parse_return() }
            Tok::Break => { self.pos += 1; Ok(Stat::Break) }
            Tok::Goto => {
                self.pos += 1;
                let name = self.expect_name("label name")?;
                Ok(Stat::Goto(name))
            }
            Tok::DoubleColon => {
                self.pos += 1;
                let name = self.expect_name("label name")?;
                self.expect(&Tok::DoubleColon, "'::'")?;
                Ok(Stat::Label(name))
            }
            _ => self.parse_expr_or_assign(),
        }
    }

    fn parse_if(&mut self) -> Result<Stat, ParseError> {
        self.pos += 1;
        let mut arms = Vec::new();
        let cond = self.parse_expr()?;
        self.expect(&Tok::Then, "'then'")?;
        let body = self.parse_block()?;
        arms.push((cond, body));
        while self.eat(&Tok::Elseif) {
            let c = self.parse_expr()?;
            self.expect(&Tok::Then, "'then'")?;
            let b = self.parse_block()?;
            arms.push((c, b));
        }
        let else_b = if self.eat(&Tok::Else) { Some(self.parse_block()?) } else { None };
        self.expect(&Tok::End, "'end'")?;
        Ok(Stat::If(arms, else_b))
    }

    fn parse_while(&mut self) -> Result<Stat, ParseError> {
        self.pos += 1;
        let cond = self.parse_expr()?;
        self.expect(&Tok::Do, "'do'")?;
        let body = self.parse_block()?;
        self.expect(&Tok::End, "'end'")?;
        Ok(Stat::While(cond, body))
    }

    fn parse_repeat(&mut self) -> Result<Stat, ParseError> {
        self.pos += 1;
        let body = self.parse_block()?;
        self.expect(&Tok::Until, "'until'")?;
        let cond = self.parse_expr()?;
        Ok(Stat::Repeat(body, cond))
    }

    fn parse_for(&mut self) -> Result<Stat, ParseError> {
        self.pos += 1;
        let first = self.expect_name("name in for")?;
        if self.eat(&Tok::Assign) {
            let a = self.parse_expr()?;
            self.expect(&Tok::Comma, "','")?;
            let b = self.parse_expr()?;
            let c = if self.eat(&Tok::Comma) { Some(self.parse_expr()?) } else { None };
            self.expect(&Tok::Do, "'do'")?;
            let body = self.parse_block()?;
            self.expect(&Tok::End, "'end'")?;
            return Ok(Stat::NumericFor(first, a, b, c, body));
        }
        // generic for
        let mut names = vec![first];
        while self.eat(&Tok::Comma) {
            names.push(self.expect_name("name in for")?);
        }
        self.expect(&Tok::In, "'in'")?;
        let mut exprs = vec![self.parse_expr()?];
        while self.eat(&Tok::Comma) { exprs.push(self.parse_expr()?); }
        self.expect(&Tok::Do, "'do'")?;
        let body = self.parse_block()?;
        self.expect(&Tok::End, "'end'")?;
        Ok(Stat::GenericFor(names, exprs, body))
    }

    fn parse_function_stat(&mut self) -> Result<Stat, ParseError> {
        self.pos += 1;
        // funcname: Name {'.' Name} [':' Name]
        let mut e = Expr::Name(self.expect_name("function name")?);
        while self.eat(&Tok::Dot) {
            let n = self.expect_name("field")?;
            e = Expr::Field(Box::new(e), n);
        }
        let mut method_name: Option<Rc<str>> = None;
        if self.eat(&Tok::Colon) {
            method_name = Some(self.expect_name("method")?);
        }
        let mut body = self.parse_funcbody()?;
        if let Some(m) = method_name {
            // add `self` as first param
            let mut params = vec![Rc::<str>::from("self")];
            params.extend(body.params.into_iter());
            body.params = params;
            e = Expr::Field(Box::new(e), m);
        }
        Ok(Stat::Assign(vec![e], vec![Expr::Function(body)]))
    }

    fn parse_local(&mut self) -> Result<Stat, ParseError> {
        self.pos += 1;
        if self.eat(&Tok::Function) {
            let name = self.expect_name("function name")?;
            let body = self.parse_funcbody()?;
            return Ok(Stat::LocalFunction(name, body));
        }
        let mut names = vec![self.expect_name("name")?];
        while self.eat(&Tok::Comma) { names.push(self.expect_name("name")?); }
        let exprs = if self.eat(&Tok::Assign) { self.parse_exp_list()? } else { Vec::new() };
        Ok(Stat::LocalAssign(names, exprs))
    }

    fn parse_return(&mut self) -> Result<Stat, ParseError> {
        // Return expr list, possibly empty
        if self.is_block_terminator() || self.is(&Tok::Semi) {
            return Ok(Stat::Return(Vec::new()));
        }
        let exprs = self.parse_exp_list()?;
        Ok(Stat::Return(exprs))
    }

    fn parse_funcbody(&mut self) -> Result<FuncBody, ParseError> {
        let line = self.line();
        self.expect(&Tok::LParen, "'('")?;
        let mut params: Vec<Rc<str>> = Vec::new();
        let mut is_vararg = false;
        if !self.is(&Tok::RParen) {
            loop {
                if self.eat(&Tok::Vararg) { is_vararg = true; break; }
                let n = self.expect_name("parameter")?;
                params.push(n);
                if !self.eat(&Tok::Comma) { break; }
            }
        }
        self.expect(&Tok::RParen, "')'")?;
        let body = self.parse_block()?;
        self.expect(&Tok::End, "'end'")?;
        Ok(FuncBody { params, is_vararg, body, line })
    }

    fn parse_exp_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut v = vec![self.parse_expr()?];
        while self.eat(&Tok::Comma) { v.push(self.parse_expr()?); }
        Ok(v)
    }

    fn parse_expr_or_assign(&mut self) -> Result<Stat, ParseError> {
        let first = self.parse_suffixed_expr()?;
        // If followed by ',' or '=', it's an assignment
        if self.is(&Tok::Comma) || self.is(&Tok::Assign) {
            let mut lhs = vec![first];
            while self.eat(&Tok::Comma) {
                lhs.push(self.parse_suffixed_expr()?);
            }
            self.expect(&Tok::Assign, "'='")?;
            let rhs = self.parse_exp_list()?;
            for e in &lhs {
                if !is_assignable(e) {
                    return Err(self.err("invalid assignment target"));
                }
            }
            return Ok(Stat::Assign(lhs, rhs));
        }
        // Must be a call statement
        if !is_call(&first) {
            return Err(self.err("syntax error: expected call or assignment"));
        }
        Ok(Stat::Call(first))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_binop(0)
    }

    fn parse_binop(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let (op, lprec, rprec) = match self.peek() {
                Tok::Or => (Some(BinOp::Or), 1, 1),
                Tok::And => (Some(BinOp::And), 2, 2),
                Tok::Lt => (Some(BinOp::Lt), 3, 3),
                Tok::Le => (Some(BinOp::Le), 3, 3),
                Tok::Gt => (Some(BinOp::Gt), 3, 3),
                Tok::Ge => (Some(BinOp::Ge), 3, 3),
                Tok::Eq => (Some(BinOp::Eq), 3, 3),
                Tok::NotEq => (Some(BinOp::NotEq), 3, 3),
                Tok::Concat => (Some(BinOp::Concat), 5, 4), // right-assoc
                Tok::Plus => (Some(BinOp::Add), 6, 6),
                Tok::Minus => (Some(BinOp::Sub), 6, 6),
                Tok::Star => (Some(BinOp::Mul), 7, 7),
                Tok::Slash => (Some(BinOp::Div), 7, 7),
                Tok::Percent => (Some(BinOp::Mod), 7, 7),
                Tok::Caret => (Some(BinOp::Pow), 10, 9), // right-assoc, binds higher than unary
                _ => (None, 0, 0),
            };
            let op = match op { Some(o) => o, None => break };
            if lprec < min_prec { break; }
            self.pos += 1;
            let right = self.parse_binop(rprec + 1)?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Tok::Not => { self.pos += 1; let e = self.parse_binop(8)?; Ok(Expr::UnOp(UnOp::Not, Box::new(e))) }
            Tok::Minus => { self.pos += 1; let e = self.parse_binop(8)?; Ok(Expr::UnOp(UnOp::Neg, Box::new(e))) }
            Tok::Hash => { self.pos += 1; let e = self.parse_binop(8)?; Ok(Expr::UnOp(UnOp::Len, Box::new(e))) }
            _ => self.parse_suffixed_expr(),
        }
    }

    fn parse_suffixed_expr(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_primary()?;
        loop {
            match self.peek() {
                Tok::Dot => {
                    self.pos += 1;
                    let name = self.expect_name("field")?;
                    e = Expr::Field(Box::new(e), name);
                }
                Tok::LBracket => {
                    self.pos += 1;
                    let k = self.parse_expr()?;
                    self.expect(&Tok::RBracket, "']'")?;
                    e = Expr::Index(Box::new(e), Box::new(k));
                }
                Tok::Colon => {
                    self.pos += 1;
                    let m = self.expect_name("method")?;
                    let args = self.parse_call_args()?;
                    e = Expr::MethodCall(Box::new(e), m, args);
                }
                Tok::LParen | Tok::Str(_) | Tok::LBrace => {
                    let args = self.parse_call_args()?;
                    e = Expr::Call(Box::new(e), args);
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        if let Tok::Str(_) = self.peek() {
            // Single-string call: f"..." -> f("...")
            if let Tok::Str(b) = self.advance() {
                return Ok(vec![Expr::Str(b)]);
            }
        }
        if self.is(&Tok::LBrace) {
            // f{...} -> f({...})
            let t = self.parse_table_constructor()?;
            return Ok(vec![t]);
        }
        self.expect(&Tok::LParen, "'('")?;
        let mut args = Vec::new();
        if !self.is(&Tok::RParen) {
            args.push(self.parse_expr()?);
            while self.eat(&Tok::Comma) { args.push(self.parse_expr()?); }
        }
        self.expect(&Tok::RParen, "')'")?;
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Tok::Nil => { self.pos += 1; Ok(Expr::Nil) }
            Tok::True => { self.pos += 1; Ok(Expr::True) }
            Tok::False => { self.pos += 1; Ok(Expr::False) }
            Tok::Number(n) => { self.pos += 1; Ok(Expr::Number(n)) }
            Tok::Str(b) => { self.pos += 1; Ok(Expr::Str(b)) }
            Tok::Vararg => { self.pos += 1; Ok(Expr::Vararg) }
            Tok::Function => { self.pos += 1; let body = self.parse_funcbody()?; Ok(Expr::Function(body)) }
            Tok::LParen => {
                self.pos += 1;
                let e = self.parse_expr()?;
                self.expect(&Tok::RParen, "')'")?;
                Ok(e)
            }
            Tok::LBrace => self.parse_table_constructor(),
            Tok::Name(n) => { self.pos += 1; Ok(Expr::Name(n)) }
            _ => Err(self.err("unexpected token in expression")),
        }
    }

    fn parse_table_constructor(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Tok::LBrace, "'{'")?;
        let mut fields = Vec::new();
        while !self.is(&Tok::RBrace) {
            let field = if self.is(&Tok::LBracket) {
                self.pos += 1;
                let k = self.parse_expr()?;
                self.expect(&Tok::RBracket, "']'")?;
                self.expect(&Tok::Assign, "'='")?;
                let v = self.parse_expr()?;
                TableField::KV(k, v)
            } else if let Tok::Name(n) = self.peek().clone() {
                if self.toks.get(self.pos + 1).map(|t| matches!(&t.tok, Tok::Assign)).unwrap_or(false) {
                    self.pos += 2;
                    let v = self.parse_expr()?;
                    TableField::KV(Expr::Str(Rc::from(n.as_bytes())), v)
                } else {
                    TableField::Array(self.parse_expr()?)
                }
            } else {
                TableField::Array(self.parse_expr()?)
            };
            fields.push(field);
            if !self.eat(&Tok::Comma) && !self.eat(&Tok::Semi) { break; }
        }
        self.expect(&Tok::RBrace, "'}'")?;
        Ok(Expr::Table(fields))
    }

    fn expect_name(&mut self, what: &str) -> Result<Rc<str>, ParseError> {
        if let Tok::Name(n) = self.peek().clone() {
            self.pos += 1;
            Ok(n)
        } else {
            Err(self.err(&format!("expected {}", what)))
        }
    }
}

fn is_assignable(e: &Expr) -> bool {
    matches!(e, Expr::Name(_) | Expr::Index(_, _) | Expr::Field(_, _))
}

fn is_call(e: &Expr) -> bool {
    matches!(e, Expr::Call(_, _) | Expr::MethodCall(_, _, _))
}
