use helion_lexer::lexer::Lexer;
use std::io::{self, Read};

fn main() {
    let mut src = String::new();
    io::stdin().read_to_string(&mut src).unwrap();

    let mut lx = Lexer::new(&src);

    loop {
        let tok = lx.next_token().unwrap();
        println!("{:?}", tok);
        if matches!(tok.kind, helion_lexer::token::TokenKind::Eof) {
            break;
        }
    }
}