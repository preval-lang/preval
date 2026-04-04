use std::{collections::HashMap, env, fs};

use ron::ser::PrettyConfig;

use crate::{
    ir::{Module, module_to_string},
    optimizations::remove_unused::remove_unused,
    parser::module::parse_module,
    tokeniser::{get_line_and_column, tokenise},
    value::{Value, primitive::IO},
    vm::{RunResult, evaluate},
};

mod ir;
mod optimizations;
mod parser;
mod tokeniser;
mod value;
mod vm;

fn main() {
    if let Some(arg1) = env::args().collect::<Vec<_>>().get(1) {
        if arg1 == "run" {
            let (module, runresult): (Module, RunResult) =
                ron::de::from_bytes(&fs::read("main.pvc").unwrap()).unwrap();

            let mut vars: HashMap<usize, Option<Value>> = HashMap::new();

            vars.insert(0, Some(Value::new(IO {})));
            vars.insert(1, Some(Value::new(IO {})));

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

                    let eval = module
                        .objects
                        .get_mut("main")
                        .unwrap()
                        .clone()
                        .data
                        .call(&module, vec![&Some(Value::new(IO)), &None]);

                    let optimized = match eval {
                        RunResult::Residualise => unreachable!(),
                        RunResult::Concrete(c) => RunResult::Concrete(c),
                        RunResult::Partial(blocks, start_block) => {
                            RunResult::Partial(remove_unused(&blocks, start_block), start_block)
                        }
                    };

                    fs::write("eval.ir", format!("{optimized:?}")).unwrap();

                    if let Some(arg1) = env::args().collect::<Vec<_>>().get(1) {
                        if arg1 == "compile" {
                            let vec = ron::ser::to_string_pretty(
                                &(module, optimized),
                                PrettyConfig::default(),
                            )
                            .unwrap();
                            fs::write("main.pvc", vec).unwrap();
                            return;
                        }
                    }

                    let mut vars: HashMap<usize, Option<Value>> = HashMap::new();

                    vars.insert(0, Some(Value::new(IO {})));
                    vars.insert(1, Some(Value::new(IO {})));

                    println!("-----");
                    println!("PASS 1 COMPLETE");
                    println!("-----");

                    run_entire_program(&module, optimized, &mut vars);
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
        RunResult::Partial(blocks, next_block) => {
            // vars.insert(1, Some(Box::new(IO {})));
            run_entire_program(module, evaluate(module, blocks, vars, next_block), vars)
        }
        RunResult::Residualise => panic!(),
    }
}
