use helion_parser::Parser;
use std::io::{self, Read};

fn main() {
    let mut src = String::new();
    io::stdin().read_to_string(&mut src).unwrap();

    match Parser::new(&src) {
        Ok(mut parser) => match parser.parse_program() {
            Ok(program) => {
                println!("{:#?}", program);
            }
            Err(e) => eprintln!("Parse error: {e}"),
        },
        Err(e) => eprintln!("Init error: {e}"),
    }
}