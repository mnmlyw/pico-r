use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Stat {
    Assign(Vec<Expr>, Vec<Expr>), // lhs..., rhs...
    LocalAssign(Vec<Rc<str>>, Vec<Expr>),
    Call(Expr), // call as statement
    Do(Block),
    While(Expr, Block),
    Repeat(Block, Expr),
    If(Vec<(Expr, Block)>, Option<Block>), // arms (cond, block) + optional else
    NumericFor(Rc<str>, Expr, Expr, Option<Expr>, Block), // name, start, stop, step, body
    GenericFor(Vec<Rc<str>>, Vec<Expr>, Block),
    LocalFunction(Rc<str>, FuncBody),
    Return(Vec<Expr>),
    Break,
    Goto(Rc<str>),
    Label(Rc<str>),
}

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub stats: Vec<Stat>,
    pub stat_lines: Vec<u32>,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Nil,
    True,
    False,
    Number(f64),
    Str(Rc<[u8]>),
    Vararg,
    Name(Rc<str>),               // Resolution to local/upvalue/global at eval time
    Index(Box<Expr>, Box<Expr>), // t[k]
    Field(Box<Expr>, Rc<str>),   // t.k
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, Rc<str>, Vec<Expr>), // obj:method(args)
    Function(FuncBody),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    Table(Vec<TableField>),
}

#[derive(Debug, Clone)]
pub enum TableField {
    Array(Expr),    // implicit numeric key
    KV(Expr, Expr), // [k]=v or k=v
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Concat,
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
    Len,
}

#[derive(Debug, Clone)]
pub struct FuncBody {
    pub params: Vec<Rc<str>>,
    pub is_vararg: bool,
    pub body: Block,
    pub line: u32,
}
