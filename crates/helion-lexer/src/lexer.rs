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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    #[test]
    fn test_keywords_and_identifiers() {
        let mut lx = Lexer::new("fn main let x const y return value");
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::KeywordFn);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Ident("main".into()));
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::KeywordLet);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Ident("x".into()));
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::KeywordConst);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Ident("y".into()));
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::KeywordReturn);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Ident("value".into()));
    }

    #[test]
    fn test_punctuation() {
        let mut lx = Lexer::new("(){+}-*/");
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::LParen);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::RParen);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::LBrace);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::RBrace);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Plus);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Minus);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Star);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Slash);
    }

    #[test]
    fn test_eof() {
        let mut lx = Lexer::new("");
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Eof);
    }
}
