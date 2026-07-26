#[derive(Debug)]
pub enum Expr {
    Ident(String),
    Number(f64),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum BinaryOp {
    Plus,
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
}

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}