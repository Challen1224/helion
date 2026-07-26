#[derive(Debug)]
pub enum Expr {
    Ident(String),
    Number(f64),
    String(String),

    UnaryMinus(Box<Expr>),
    UnaryBang(Box<Expr>),

    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    // Arithmetic
    Plus,
    Minus,
    Star,
    Slash,

    // Comparison
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Equality
    EqualEqual,
    BangEqual,
}

#[derive(Debug)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Return {
        value: Expr,
    },
    ExprStmt {
        expr: Expr,
    },
    Block {
        stmts: Vec<Stmt>,
    },
    Function {
        name: String,
        body: Box<Stmt>, // MUST be boxed
    },
}

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}