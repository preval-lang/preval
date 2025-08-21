use std::fs;

use crate::{
    // compiler::compile,
    module_parser::parse_module,
    tokeniser::{get_line_and_column, tokenise},
    vm::run,
};

// mod compiler;
mod expression_parser;
mod ir;
mod module_parser;
mod tokeniser;
mod vm;

fn main() {
    println!("TODO: enforce returns being the last expression");

    let file = "main.pv";
    let src = String::from_utf8(fs::read(file).unwrap()).unwrap();
    let tokens = tokenise(&src, 0);
    match tokens {
        Err(err) => {
            let (line, column) = get_line_and_column(&src, err.idx).unwrap();
            eprintln!("{:?} at {file}:{line}:{column}", err.error);
        }
        Ok(tokens) => {
            let module = parse_module(&tokens);
            match module {
                Ok(module) => {
                    fs::write("out.s", format!("{module:#?}")).unwrap();

                    run(&module, module.functions.get("main").unwrap());
                }
                Err(err) => {
                    let (line, column) = get_line_and_column(&src, err.idx).unwrap();
                    eprintln!("ParseError: {:?} at {file}:{line}:{column}", err.error);
                }
            }
        }
    }
}
