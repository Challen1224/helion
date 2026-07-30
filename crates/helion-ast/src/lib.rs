#[derive(Debug, Clone)]
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

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    // ⭐ Array literal: [expr, expr, ...]
    Array(Vec<Expr>),

    // ⭐ Indexing: array[index]
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },

    // ⭐ Object literal: { key: expr, key2: expr }
    Object(Vec<(String, Expr)>),

    // ⭐ Property access: object.key
    Property {
        object: Box<Expr>,
        property: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Plus,
    Minus,
    Star,
    Slash,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    BangEqual,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Assign {
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
        params: Vec<String>,
        body: Box<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },

    // ⭐ Array element assignment: array[index] = value
    ArrayAssign {
        array: Expr,
        index: Expr,
        value: Expr,
    },

    // ⭐ Object property assignment: object.key = value
    ObjectAssign {
        object: Expr,
        property: String,
        value: Expr,
    },
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub params: Vec<String>,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}