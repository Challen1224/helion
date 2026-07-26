use helion_ast::{BinaryOp, Expr, Program, Stmt};
use helion_lexer::lexer::{Lexer, LexError};
use helion_lexer::token::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token {token:?} at {line}:{column}")]
    UnexpectedToken { token: Token, line: usize, column: usize },

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
        Ok(Self { lexer, current: first })
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next_token()?;
        Ok(())
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
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

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();

        while self.current.kind != TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
        }

        Ok(Program { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match &self.current.kind {
            TokenKind::KeywordLet => self.parse_let(),
            TokenKind::KeywordReturn => self.parse_return(),
            _ => Err(ParseError::UnexpectedToken {
                token: self.current.clone(),
                line: self.current.line,
                column: self.current.column,
            }),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        // let <ident> = <expr>
        self.advance()?; // consume 'let'

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

        self.expect(TokenKind::Equal)?;

        let value = self.parse_expr()?;

        Ok(Stmt::Let { name, value })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        // return <expr>
        self.advance()?; // consume 'return'
        let value = self.parse_expr()?;
        Ok(Stmt::Return { value })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;

        while let TokenKind::Plus = self.current.kind {
            self.advance()?; // consume '+'
            let right = self.parse_primary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Plus,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
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