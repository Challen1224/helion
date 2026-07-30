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

    fn peek_next_is(&mut self, kind: TokenKind) -> Result<bool, ParseError> {
        let tok = self.lexer.peek_token()?;
        Ok(tok.kind == kind)
    }

    // PROGRAM

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

    // FUNCTIONS

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
        self.expect(TokenKind::LParen)?;

        let mut params = Vec::new();

        self.skip_ws()?;
        if self.current.kind != TokenKind::RParen {
            loop {
                match &self.current.kind {
                    TokenKind::Ident(s) => {
                        params.push(s.clone());
                        self.advance()?;
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            token: self.current.clone(),
                            line: self.current.line,
                            column: self.current.column,
                        })
                    }
                }

                self.skip_ws()?;
                if self.current.kind == TokenKind::Comma {
                    self.advance()?;
                    self.skip_ws()?;
                    continue;
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen)?;
        self.skip_ws()?;

        let body = self.parse_block()?;

        Ok(Stmt::Function {
            name,
            params,
            body: Box::new(body),
        })
    }

    // BLOCKS

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

    // STATEMENTS

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.skip_ws()?;

        match self.current.kind {
            TokenKind::KeywordLet => self.parse_let(),
            TokenKind::KeywordReturn => self.parse_return(),
            TokenKind::KeywordWhile => self.parse_while(),
            TokenKind::KeywordIf => self.parse_if(),
            TokenKind::LBrace => self.parse_block(),

            TokenKind::Ident(_) => {
                let mut clone = self.clone_for_peek();
                clone.advance()?; // move past ident

                // ⭐ Array assignment
                if clone.current.kind == TokenKind::LBracket {
                    return self.parse_assign();
                }

                // ⭐ Object property assignment: obj.key = value
                if clone.current.kind == TokenKind::Dot {
                    return self.parse_object_assign();
                }

                // ⭐ Normal assignment
                if self.peek_next_is(TokenKind::Equal)? {
                    return self.parse_assign();
                }

                let expr = self.parse_expr()?;
                Ok(Stmt::ExprStmt { expr })
            }

            _ => {
                let expr = self.parse_expr()?;
                Ok(Stmt::ExprStmt { expr })
            }
        }
    }

    fn clone_for_peek(&self) -> Parser<'a> {
        Parser {
            lexer: self.lexer.clone(),
            current: self.current.clone(),
        }
    }

    // ⭐ ARRAY ASSIGNMENT SUPPORT

    fn parse_assign(&mut self) -> Result<Stmt, ParseError> {
        let mut target = match &self.current.kind {
            TokenKind::Ident(s) => {
                let n = s.clone();
                self.advance()?;
                Expr::Ident(n)
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
        while self.current.kind == TokenKind::LBracket {
            self.advance()?; // '['
            let index_expr = self.parse_expr()?;
            self.expect(TokenKind::RBracket)?;
            target = Expr::Index {
                array: Box::new(target),
                index: Box::new(index_expr),
            };
            self.skip_ws()?;
        }

        self.expect(TokenKind::Equal)?;
        self.skip_ws()?;

        let value = self.parse_expr()?;

        if let Expr::Index { array, index } = target {
            return Ok(Stmt::ArrayAssign {
                array: *array,
                index: *index,
                value,
            });
        }

        if let Expr::Ident(name) = target {
            return Ok(Stmt::Assign { name, value });
        }

        Err(ParseError::UnexpectedToken {
            token: self.current.clone(),
            line: self.current.line,
            column: self.current.column,
        })
    }

    // ⭐ OBJECT PROPERTY ASSIGNMENT: obj.key = value

    fn parse_object_assign(&mut self) -> Result<Stmt, ParseError> {
        let obj_name = match &self.current.kind {
            TokenKind::Ident(s) => s.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: self.current.clone(),
                    line: self.current.line,
                    column: self.current.column,
                })
            }
        };

        self.advance()?; // consume ident
        self.skip_ws()?;

        self.expect(TokenKind::Dot)?;
        self.skip_ws()?;

        let property = match &self.current.kind {
            TokenKind::Ident(s) => s.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: self.current.clone(),
                    line: self.current.line,
                    column: self.current.column,
                })
            }
        };

        self.advance()?; // consume property
        self.skip_ws()?;

        self.expect(TokenKind::Equal)?;
        self.skip_ws()?;

        let value = self.parse_expr()?;

        Ok(Stmt::ObjectAssign {
            object: Expr::Ident(obj_name),
            property,
            value,
        })
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

    // EXPRESSIONS

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

    // ⭐ PRIMARY: identifiers, calls, indexing, literals, arrays, objects, properties

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.skip_ws()?;

        match &self.current.kind {
            // IDENT or CALL or INDEX or PROPERTY
            TokenKind::Ident(s) => {
                let name = s.clone();
                self.advance()?; // consume ident

                self.skip_ws()?;
                let mut expr = Expr::Ident(name);

                // ⭐ Parse call: foo(...)
                if self.current.kind == TokenKind::LParen {
                    self.advance()?; // consume '('
                    let mut args = Vec::new();

                    self.skip_ws()?;
                    if self.current.kind != TokenKind::RParen {
                        loop {
                            let arg = self.parse_expr()?;
                            args.push(arg);
                            self.skip_ws()?;
                            if self.current.kind == TokenKind::Comma {
                                self.advance()?;
                                self.skip_ws()?;
                                continue;
                            } else {
                                break;
                            }
                        }
                    }

                    self.expect(TokenKind::RParen)?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }

                // ⭐ Parse property access: foo.bar
                loop {
                    self.skip_ws()?;
                    if self.current.kind == TokenKind::Dot {
                        self.advance()?; // consume '.'
                        self.skip_ws()?;

                        let prop = match &self.current.kind {
                            TokenKind::Ident(s) => s.clone(),
                            _ => {
                                return Err(ParseError::UnexpectedToken {
                                    token: self.current.clone(),
                                    line: self.current.line,
                                    column: self.current.column,
                                })
                            }
                        };

                        self.advance()?; // consume property
                        expr = Expr::Property {
                            object: Box::new(expr),
                            property: prop,
                        };
                        continue;
                    }

                    // ⭐ Parse indexing: foo[expr]
                    if self.current.kind == TokenKind::LBracket {
                        self.advance()?; // consume '['
                        let index_expr = self.parse_expr()?;
                        self.expect(TokenKind::RBracket)?;
                        expr = Expr::Index {
                            array: Box::new(expr),
                            index: Box::new(index_expr),
                        };
                        continue;
                    }

                    break;
                }

                return Ok(expr);
            }

            // NUMBER
            TokenKind::Number(n) => {
                let v = *n;
                self.advance()?;
                return Ok(Expr::Number(v));
            }

            // STRING
            TokenKind::String(s) => {
                let v = s.clone();
                self.advance()?;
                return Ok(Expr::String(v));
            }

            // PAREN GROUPING
            TokenKind::LParen => {
                self.advance()?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                return Ok(expr);
            }

                        // ⭐ ARRAY LITERAL: [expr, expr, ...]
            TokenKind::LBracket => {
                self.advance()?; // consume '['
                let mut items = Vec::new();

                self.skip_ws()?;
                if self.current.kind != TokenKind::RBracket {
                    loop {
                        let item = self.parse_expr()?;
                        items.push(item);
                        self.skip_ws()?;
                        if self.current.kind == TokenKind::Comma {
                            self.advance()?;
                            self.skip_ws()?;
                            continue;
                        } else {
                            break;
                        }
                    }
                }

                self.expect(TokenKind::RBracket)?;
                return Ok(Expr::Array(items));
            }

            // ⭐ OBJECT LITERAL: { key: value, key2: value }
            TokenKind::LBrace => {
                self.advance()?; // consume '{'
                let mut fields = Vec::new();

                self.skip_ws()?;
                if self.current.kind != TokenKind::RBrace {
                    loop {
                        // key must be identifier
                        let key = match &self.current.kind {
                            TokenKind::Ident(s) => s.clone(),
                            _ => {
                                return Err(ParseError::UnexpectedToken {
                                    token: self.current.clone(),
                                    line: self.current.line,
                                    column: self.current.column,
                                })
                            }
                        };

                        self.advance()?; // consume key
                        self.skip_ws()?;

                        // expect ':'
                        self.expect(TokenKind::Colon)?;
                        self.skip_ws()?;

                        // parse value
                        let value = self.parse_expr()?;
                        fields.push((key, value));

                        self.skip_ws()?;
                        if self.current.kind == TokenKind::Comma {
                            self.advance()?;
                            self.skip_ws()?;
                            continue;
                        } else {
                            break;
                        }
                    }
                }

                self.expect(TokenKind::RBrace)?;
                return Ok(Expr::Object(fields));
            }

            _ => Err(ParseError::UnexpectedToken {
                token: self.current.clone(),
                line: self.current.line,
                column: self.current.column,
            }),
        }
    }
}
            