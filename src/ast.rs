#![allow(dead_code)]

pub enum Expr {
    Identifier(String),
    Literal(Literal),
    This,

    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Assign { target: Box<Expr>, value: Box<Expr> },
    Ternary { cond: Box<Expr>, then_: Box<Expr>, else_: Box<Expr> },

    Member { object: Box<Expr>, property: String }, // a.b
    Index { object: Box<Expr>, index: Box<Expr> },  // a[b]
    Call { callee: Box<Expr>, args: Vec<Expr> },    // f(x)
    New { callee: Box<Expr>, args: Vec<Expr> },     // new F(x)
}

pub enum Literal {
    Null,
    Undefined,
    Bool(bool),
    Number(f64),
    String(String),
}

pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr, UShr,
}

pub enum UnaryOp {
    Neg, Not, BitNot, Typeof, Void, Delete,
    PreInc, PreDec, PostInc, PostDec,
}

pub enum Stmt {
    Block(Vec<Stmt>),
    Var(Vec<(String, Option<Expr>)>),
    Empty,
    Expr(Expr),
    If { cond: Expr, then_: Box<Stmt>, else_: Option<Box<Stmt>> },
    While { cond: Expr, body: Box<Stmt> },
    For { init: Option<ForInit>, cond: Option<Expr>, update: Option<Expr>, body: Box<Stmt> },
    ForIn { var: String, expr: Expr, body: Box<Stmt> },
    Continue,
    Break,
    Return(Option<Expr>),
    With { expr: Expr, body: Box<Stmt> },
    Function(Function),
}

pub enum ForInit {
    Var(Vec<(String, Option<Expr>)>),
    Expr(Expr),
}

pub struct Function {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

pub struct Program {
    pub body: Vec<Stmt>,
}
