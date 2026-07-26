pub mod token;
pub mod lexer;
#[cfg(test)]
mod tests {
    use super::lexer::Lexer;
    use super::token::TokenKind;

    #[test]
    fn basic_punct() {
        let mut lx = Lexer::new("(){}+");
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::LParen);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::RParen);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::LBrace);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::RBrace);
        assert_eq!(lx.next_token().unwrap().kind, TokenKind::Plus);
    }
}