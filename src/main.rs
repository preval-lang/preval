use std::{env, fs};

use crate::{
    // compiler::compile,
    ir::{Module, to_string},
    module_parser::parse_module,
    tokeniser::{get_line_and_column, tokenise},
    vm::{RunResult, evaluate, run},
};

// mod compiler;
mod builtins;
mod expression_parser;
mod ir;
mod module_parser;
mod tokeniser;
mod typ;
mod vm;

fn main() {
    if let Some(arg1) = env::args().collect::<Vec<_>>().get(1) {
        if arg1 == "run" {
            let (module, runresult): (Module, RunResult) =
                postcard::from_bytes(&fs::read("main.pvc").unwrap()).unwrap();

            run_entire_program(&module, runresult);
            return;
        }
    }

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

                    let eval = run(&mut module, main, vec![Some(Vec::new()), None]);

                    if let Some(arg1) = env::args().collect::<Vec<_>>().get(1) {
                        if arg1 == "compile" {
                            let vec = postcard::to_stdvec(&(module, eval)).unwrap();
                            fs::write("main.pvc", vec).unwrap();
                            return;
                        }
                    }

                    // fs::write(
                    //     "eval.ir",
                    //     match eval.clone() {
                    //         RunResult::Concrete(_) => todo!(),
                    //         RunResult::Partial(blocks, vars) => {
                    //             to_string(&module, &blocks, vars, 0)
                    //         },
                    //         RunResult::ConditionalPartial { condition, then, els } => {

                    //         }
                    //     },
                    // )
                    // .unwrap();

                    run_entire_program(&module, eval);
                }
                Err(err) => {
                    let (line, column) = get_line_and_column(&src, err.idx).unwrap();
                    eprintln!("ParseError: {:?} at {file}:{line}:{column}", err.error);
                }
            }
        }
    }
}

fn run_entire_program(module: &Module, eval: RunResult) -> bool {
    match eval {
        RunResult::Concrete(_) => false,
        RunResult::Partial(blocks, mut vars) => {
            vars.insert(1, Some(Vec::new()));
            run_entire_program(module, evaluate(module, blocks, &mut vars, 0))
        }
        RunResult::ConditionalPartial {
            condition,
            then,
            els,
        } => {
            let (cond_blocks, mut cond_vars) = condition;
            cond_vars.insert(1, Some(Vec::new()));
            run_entire_program(
                module,
                if run_entire_program(module, evaluate(module, cond_blocks, &mut cond_vars, 0)) {
                    *then
                } else {
                    *els
                },
            )
        }
        RunResult::ThenElseJump(bool) => bool,
    }
}
