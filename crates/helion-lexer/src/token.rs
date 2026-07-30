#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    KeywordFn,
    KeywordLet,
    KeywordConst,
    KeywordReturn,
    KeywordIf,
    KeywordElse,
    KeywordWhile,

    // Identifiers & literals
    Ident(String),
    Number(f64),
    String(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,

    // ⭐ NEW TOKENS FOR OBJECTS
    Colon,   // :
    Dot,     // .

    // End of file
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}