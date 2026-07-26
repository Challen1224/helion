use crate::token::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character '{ch}' at {line}:{column}")]
    UnexpectedChar { ch: char, line: usize, column: usize },
}

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    current: Option<char>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        // Strip UTF‑8 BOM if present
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

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        while let Some(c) = self.current {
            match c {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                    continue;
                }
                '(' => {
                    self.advance();
                    return Ok(self.make_token(TokenKind::LParen));
                }
                ')' => {
                    self.advance();
                    return Ok(self.make_token(TokenKind::RParen));
                }
                '{' => {
                    self.advance();
                    return Ok(self.make_token(TokenKind::LBrace));
                }
                '}' => {
                    self.advance();
                    return Ok(self.make_token(TokenKind::RBrace));
                }
                '+' => {
                    self.advance();
                    return Ok(self.make_token(TokenKind::Plus));
                }
                '-' => {
                    self.advance();
                    return Ok(self.make_token(TokenKind::Minus));
                }
                '*' => {
                    self.advance();
                    return Ok(self.make_token(TokenKind::Star));
                }
                '/' => {
                    self.advance();
                    return Ok(self.make_token(TokenKind::Slash));
                }
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