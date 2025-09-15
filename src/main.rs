use std::fs;

use crate::{
    // compiler::compile,
    module_parser::parse_module,
    tokeniser::{get_line_and_column, tokenise},
    vm::{RunResult, evaluate, run},
};

// mod compiler;
mod expression_parser;
mod ir;
mod module_parser;
mod tokeniser;
mod typ;
mod vm;

fn main() {
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
                Ok(mut module) => {
                    fs::write("ir.ir", format!("{module:#?}")).unwrap();

                    let mut main = module.functions.remove("main").unwrap();

                    let eval = run(&module, main, vec![Some(Vec::new()), None]);

                    fs::write("eval.ir", format!("{eval:#?}")).unwrap();

                    match eval {
                        RunResult::Partial(blocks, mut vars) => {
                            vars.insert(1, Some(Vec::new()));
                            evaluate(&module, blocks, &mut vars);
                        }
                        _ => {}
                    }
                }
                Err(err) => {
                    let (line, column) = get_line_and_column(&src, err.idx).unwrap();
                    eprintln!("ParseError: {:?} at {file}:{line}:{column}", err.error);
                }
            }
        }
    }
}
