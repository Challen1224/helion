use helion_ast::{BinaryOp, Expr, Program, Stmt};
use helion_lexer::lexer::{LexError, Lexer};
use helion_lexer::token::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token {token:?} at {line}:{column}")]
    UnexpectedToken {
        token: Token,
        line: usize,
        column: usize,
    },

    #[error("Lexer error: {0}")]
    Lex(#[from] LexError),
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(src);
        let first = lexer.next_token()?;
        Ok(Self {
            lexer,
            current: first,
        })
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next_token()?;
        Ok(())
    }

    // Your lexer only emits semicolons as "noise".
    fn skip_ws(&mut self) -> Result<(), ParseError> {
        while self.current.kind == TokenKind::Semicolon {
            self.advance()?;
        }
        Ok(())
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        self.skip_ws()?;
        if self.current.kind == kind {
            self.advance()
        } else {
            Err(ParseError::UnexpectedToken {
                token: self.current.clone(),
                line: self.current.line,
                column: self.current.column,
            })
        }
    }

    // ============================================
    // PROGRAM
    // ============================================

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();

        self.skip_ws()?;

        while self.current.kind != TokenKind::Eof {
            stmts.push(self.parse_top_level()?);
            self.skip_ws()?;
        }

        Ok(Program { stmts })
    }

    fn parse_top_level(&mut self) -> Result<Stmt, ParseError> {
        self.skip_ws()?;

        match self.current.kind {
            TokenKind::KeywordFn => self.parse_function(),
            TokenKind::KeywordLet => self.parse_let(),
            TokenKind::KeywordReturn => self.parse_return(),
            TokenKind::LBrace => self.parse_block(),
            _ => self.parse_stmt(),
        }
    }

    // ============================================
    // FUNCTIONS
    // ============================================

    fn parse_function(&mut self) -> Result<Stmt, ParseError> {
        self.advance()?; // consume "fn"
        self.skip_ws()?;

        let name = match &self.current.kind {
            TokenKind::Ident(s) => {
                let n = s.clone();
                self.advance()?;
                n
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: self.current.clone(),
                    line: self.current.line,
                    column: self.current.column,
                })
            }
        };

        self.skip_ws()?;
        let body = self.parse_block()?;

        Ok(Stmt::Function {
            name,
            body: Box::new(body),
        })
    }

    // ============================================
    // BLOCKS
    // ============================================

    fn parse_block(&mut self) -> Result<Stmt, ParseError> {
        self.skip_ws()?;
        self.expect(TokenKind::LBrace)?;

        let mut stmts = Vec::new();
        self.skip_ws()?;

        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
            self.skip_ws()?;
        }

        self.expect(TokenKind::RBrace)?;
        Ok(Stmt::Block { stmts })
    }

    // ============================================
    // STATEMENTS
    // ============================================

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.skip_ws()?;

        match self.current.kind {
            TokenKind::KeywordLet => self.parse_let(),
            TokenKind::KeywordReturn => self.parse_return(),
            TokenKind::KeywordWhile => self.parse_while(),
            TokenKind::KeywordIf => self.parse_if(),
            TokenKind::LBrace => self.parse_block(),
            _ => {
                let expr = self.parse_expr()?;
                Ok(Stmt::ExprStmt { expr })
            }
        }
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.advance()?; // consume "while"
        self.skip_ws()?;

        let condition = self.parse_expr()?;
        self.skip_ws()?;

        let body = self.parse_block()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.advance()?; // consume "if"
        self.skip_ws()?;

        let condition = self.parse_expr()?;
        self.skip_ws()?;

        let then_branch = self.parse_block()?;
        self.skip_ws()?;

        let else_branch = if self.current.kind == TokenKind::KeywordElse {
            self.advance()?; // consume "else"
            self.skip_ws()?;
            Some(Box::new(self.parse_block()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        self.advance()?; // consume 'let'
        self.skip_ws()?;

        let name = match &self.current.kind {
            TokenKind::Ident(s) => {
                let n = s.clone();
                self.advance()?;
                n
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: self.current.clone(),
                    line: self.current.line,
                    column: self.current.column,
                })
            }
        };

        self.skip_ws()?;
        self.expect(TokenKind::Equal)?;

        self.skip_ws()?;
        let value = self.parse_expr()?;

        Ok(Stmt::Let { name, value })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.advance()?; // consume 'return'
        self.skip_ws()?;

        let value = self.parse_expr()?;
        Ok(Stmt::Return { value })
    }

    // ============================================
    // EXPRESSIONS
    // ============================================

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.skip_ws()?;
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_comparison()?;

        loop {
            match self.current.kind {
                TokenKind::EqualEqual => {
                    self.advance()?;
                    let right = self.parse_comparison()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::EqualEqual,
                        right: Box::new(right),
                    };
                }
                TokenKind::BangEqual => {
                    self.advance()?;
                    let right = self.parse_comparison()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::BangEqual,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_additive()?;

        loop {
            match self.current.kind {
                TokenKind::Less => {
                    self.advance()?;
                    let right = self.parse_additive()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::Less,
                        right: Box::new(right),
                    };
                }
                TokenKind::LessEqual => {
                    self.advance()?;
                    let right = self.parse_additive()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::LessEqual,
                        right: Box::new(right),
                    };
                }
                TokenKind::Greater => {
                    self.advance()?;
                    let right = self.parse_additive()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::Greater,
                        right: Box::new(right),
                    };
                }
                TokenKind::GreaterEqual => {
                    self.advance()?;
                    let right = self.parse_additive()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::GreaterEqual,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            match self.current.kind {
                TokenKind::Plus => {
                    self.advance()?;
                    let right = self.parse_multiplicative()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::Plus,
                        right: Box::new(right),
                    };
                }
                TokenKind::Minus => {
                    self.advance()?;
                    let right = self.parse_multiplicative()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::Minus,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;

        loop {
            match self.current.kind {
                TokenKind::Star => {
                    self.advance()?;
                    let right = self.parse_unary()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::Star,
                        right: Box::new(right),
                    };
                }
                TokenKind::Slash => {
                    self.advance()?;
                    let right = self.parse_unary()?;
                    expr = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::Slash,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        self.skip_ws()?;

        match self.current.kind {
            TokenKind::Minus => {
                self.advance()?;
                let right = self.parse_unary()?;
                Ok(Expr::UnaryMinus(Box::new(right)))
            }
            TokenKind::Bang => {
                self.advance()?;
                let right = self.parse_unary()?;
                Ok(Expr::UnaryBang(Box::new(right)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.skip_ws()?;

        match &self.current.kind {
            TokenKind::Ident(s) => {
                let name = s.clone();
                self.advance()?;
                Ok(Expr::Ident(name))
            }
            TokenKind::Number(n) => {
                let v = *n;
                self.advance()?;
                Ok(Expr::Number(v))
            }
            TokenKind::String(s) => {
                let v = s.clone();
                self.advance()?;
                Ok(Expr::String(v))
            }
            TokenKind::LParen => {
                self.advance()?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken {
                token: self.current.clone(),
                line: self.current.line,
                column: self.current.column,
            }),
        }
    }
}