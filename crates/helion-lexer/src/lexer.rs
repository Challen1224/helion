use crate::token::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character '{ch}' at {line}:{column}")]
    UnexpectedChar { ch: char, line: usize, column: usize },

    #[error("Invalid number literal at {line}:{column}")]
    InvalidNumber { line: usize, column: usize },
}

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        let src = src.trim_start_matches('\u{feff}');
        let mut chars = src.chars();
        let current = chars.next();

        Self {
            chars,
            current,
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) {
        if let Some(c) = self.current {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.current = self.chars.next();
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            line: self.line,
            column: self.column,
        }
    }

    fn lex_identifier(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.column;

        let mut ident = String::new();

        while let Some(c) = self.current {
            match c {
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    ident.push(c);
                    self.advance();
                }
                _ => break,
            }
        }

        let kind = match ident.as_str() {
            "fn" => TokenKind::KeywordFn,
            "let" => TokenKind::KeywordLet,
            "const" => TokenKind::KeywordConst,
            "return" => TokenKind::KeywordReturn,
            _ => TokenKind::Ident(ident),
        };

        Token {
            kind,
            line: start_line,
            column: start_col,
        }
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        let mut num = String::new();
        let mut has_dot = false;

        while let Some(c) = self.current {
            match c {
                '0'..='9' => {
                    num.push(c);
                    self.advance();
                }
                '.' if !has_dot => {
                    has_dot = true;
                    num.push('.');
                    self.advance();
                }
                _ => break,
            }
        }

        match num.parse::<f64>() {
            Ok(value) => Ok(Token {
                kind: TokenKind::Number(value),
                line: start_line,
                column: start_col,
            }),
            Err(_) => Err(LexError::InvalidNumber {
                line: start_line,
                column: start_col,
            }),
        }
    }

    fn lex_operator(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        match self.current {
            Some('=') => {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token { kind: TokenKind::EqualEqual, line: start_line, column: start_col });
                }
                return Ok(Token { kind: TokenKind::Equal, line: start_line, column: start_col });
            }

            Some('!') => {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token { kind: TokenKind::BangEqual, line: start_line, column: start_col });
                }
                return Ok(Token { kind: TokenKind::Bang, line: start_line, column: start_col });
            }

            Some('<') => {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token { kind: TokenKind::LessEqual, line: start_line, column: start_col });
                }
                return Ok(Token { kind: TokenKind::Less, line: start_line, column: start_col });
            }

            Some('>') => {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token { kind: TokenKind::GreaterEqual, line: start_line, column: start_col });
                }
                return Ok(Token { kind: TokenKind::Greater, line: start_line, column: start_col });
            }

            _ => Err(LexError::UnexpectedChar {
                ch: self.current.unwrap(),
                line: start_line,
                column: start_col,
            }),
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        while let Some(c) = self.current {
            match c {
                // Whitespace
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                    continue;
                }

                // Identifiers + keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    return Ok(self.lex_identifier());
                }

                // Numbers
                '0'..='9' => {
                    return self.lex_number();
                }

                // Operators
                '=' | '!' | '<' | '>' => {
                    return self.lex_operator();
                }

                // Punctuation
                '(' => { self.advance(); return Ok(self.make_token(TokenKind::LParen)); }
                ')' => { self.advance(); return Ok(self.make_token(TokenKind::RParen)); }
                '{' => { self.advance(); return Ok(self.make_token(TokenKind::LBrace)); }
                '}' => { self.advance(); return Ok(self.make_token(TokenKind::RBrace)); }
                '+' => { self.advance(); return Ok(self.make_token(TokenKind::Plus)); }
                '-' => { self.advance(); return Ok(self.make_token(TokenKind::Minus)); }
                '*' => { self.advance(); return Ok(self.make_token(TokenKind::Star)); }
                '/' => { self.advance(); return Ok(self.make_token(TokenKind::Slash)); }

                // Unknown character
                _ => {
                    return Err(LexError::UnexpectedChar {
                        ch: c,
                        line: self.line,
                        column: self.column,
                    });
                }
            }
        }

        Ok(self.make_token(TokenKind::Eof))
    }
}