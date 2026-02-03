#[derive(PartialEq, Debug)]
pub enum Expr {
    Identifier(String),
    Literal(Literal),
    This,
    AssignOp(AssignOp),
    Empty,

    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Update {
        op: UpdateOp,
        prefix: bool, // true = ++x, false = x++
        argument: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Assign {
        target: Box<Expr>,
        op: AssignOp,
        value: Box<Expr>,
    },
    Ternary {
        cond: Box<Expr>,
        then_: Box<Expr>,
        else_: Box<Expr>,
    },
    Member {
        object: Box<Expr>,
        property: String,
    }, // a.b
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    }, // a[b]
    Call {
        callee: Box<Expr>,
        args: Box<Expr>,
    }, // f(x)
    New {
        callee: Box<Expr>,
        args: Box<Expr>,
    }, // new F(x)
    Sequence(Vec<Expr>), // example: 1, 2, 3;
    Function(Function),
}

#[derive(PartialEq, Debug)]
pub enum Literal {
    Null,
    Undefined,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Expr>),
    Object(Vec<(PropertyKey, Expr)>), // { a: 1, b: 2 }
}

#[derive(PartialEq, Debug)]
pub enum PropertyKey {
    Identifier(String),
    String(String),
    Number(f64),
}

#[derive(PartialEq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    UShr,
    In,
}

#[derive(PartialEq, Debug)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
    BitNot,
    Typeof,
    Void,
    Delete,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}

#[derive(PartialEq, Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Var(Vec<(String, Option<Expr>)>),
    Empty,
    Expr(Expr),
    If {
        cond: Expr,
        then_: Box<Stmt>,
        else_: Option<Box<Stmt>>,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
    For {
        init: Option<ForInit>,
        cond: Option<Expr>,
        update: Option<Expr>,
        body: Box<Stmt>,
    },
    ForIn {
        var: String,
        expr: Expr,
        body: Box<Stmt>,
    },
    Continue,
    Break,
    Return(Option<Expr>),
    With {
        expr: Expr,
        body: Box<Stmt>,
    },
    Function(Function),
}

#[derive(PartialEq, Debug)]
pub enum ForInit {
    Var(Vec<(String, Option<Expr>)>),
    Expr(Expr),
}

#[derive(PartialEq, Debug)]
pub struct Function {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(PartialEq, Debug)]
pub struct Program {
    pub body: Vec<Stmt>,
}

#[derive(PartialEq, Debug)]
pub enum AssignOp {
    Assign,       // =
    AddAssign,    // +=
    SubAssign,    // -=
    MulAssign,    // *=
    DivAssign,    // /=
    ModAssign,    // %=
    ShlAssign,    // <<=
    ShrAssign,    // >>=
    UShrAssign,   // >>>=
    BitAndAssign, // &=
    BitOrAssign,  // |=
    BitXorAssign, // ^=
}

#[derive(PartialEq, Debug)]
pub enum UpdateOp {
    Inc, // ++
    Dec, // --
}
