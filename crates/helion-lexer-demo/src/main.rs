use helion_parser::Parser;
use helion_interpreter::Interpreter;
use std::io::{self, Read};

fn main() {
    let mut src = String::new();
    io::stdin().read_to_string(&mut src).unwrap();

    let mut parser = Parser::new(&src).unwrap();
    let program = parser.parse_program().unwrap();

    let interp = Interpreter;
    let result = interp.run(&program).unwrap();

    println!("Result: {:?}", result);
}