use std::{collections::HashMap, env, fs};

use crate::{
    // compiler::compile,
    ir::{Module, module_to_string, to_string},
    module_parser::parse_module,
    tokeniser::{get_line_and_column, tokenise},
    vm::{RunResult, evaluate},
};

use value::Value;

// mod compiler;
mod builtins;
mod expression_parser;
mod ir;
mod module_parser;
mod tokeniser;
mod typ;
mod value;
mod vm;

fn main() {
    if let Some(arg1) = env::args().collect::<Vec<_>>().get(1) {
        if arg1 == "run" {
            let mut vars = HashMap::new();

            vars.insert(0, Some(Value::IO));
            vars.insert(1, Some(Value::IO));

            let (module, runresult): (Module, RunResult) =
                postcard::from_bytes(&fs::read("main.pvc").unwrap()).unwrap();

            run_entire_program(&module, runresult, &mut vars);
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
                    fs::write("ir.ir", module_to_string(&module)).unwrap();

                    let mut vars = HashMap::new();

                    vars.insert(0, Some(Value::IO));
                    vars.insert(1, None);

                    let eval = evaluate(&module, module.functions["main"].ir.clone(), &mut vars, 0);

                    if let Some(arg1) = env::args().collect::<Vec<_>>().get(1) {
                        if arg1 == "compile" {
                            let vec = postcard::to_stdvec(&(module, eval)).unwrap();
                            fs::write("main.pvc", vec).unwrap();
                            return;
                        }
                    }

                    fs::write(
                        "eval.ir",
                        match eval.clone() {
                            RunResult::Concrete(value) => format!("{value}"),
                            RunResult::Partial(blocks, _) => to_string(&blocks, 0),
                        },
                    )
                    .unwrap();

                    vars.insert(1, Some(Value::IO));

                    run_entire_program(&module, eval, &mut vars);
                }
                Err(err) => {
                    let (line, column) = get_line_and_column(&src, err.idx).unwrap();
                    eprintln!("ParseError: {:?} at {file}:{line}:{column}", err.error);
                }
            }
        }
    }
}

fn run_entire_program(
    module: &Module,
    eval: RunResult,
    vars: &mut HashMap<usize, Option<Value>>,
) -> bool {
    match eval {
        RunResult::Concrete(_) => false,
        RunResult::Partial(blocks, _) => {
            vars.insert(1, Some(Value::IO));
            run_entire_program(module, evaluate(module, blocks, vars, 0), vars)
        }
    }
}
