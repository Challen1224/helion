use helion_parser::Parser;
use helion_interpreter::Interpreter;
use std::fs;

fn main() {
    println!("USING INTERPRETER: {:?}", std::any::type_name::<Interpreter>());
    println!("HELION EXECUTED");

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: helion <file>");
        return;
    }

    let src = match fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read file: {e}");
            return;
        }
    };

    let mut parser = match Parser::new(&src) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse init error: {e}");
            return;
        }
    };

    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {e}");
            return;
        }
    };

    println!("PROGRAM: {:#?}", program);

    let interp = Interpreter;
    match interp.run(&program) {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => eprintln!("Runtime error: {e}"),
    }
}