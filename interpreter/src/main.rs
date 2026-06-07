use std::{borrow::Cow, collections::HashMap, env, fs};

use preval_lib::{
    ir::Partial,
    parser::{module::parse_module, typ::InfoTypeExpr},
    passes::remove_unused::{Usage, remove_unused},
    tokeniser::{get_line_and_column, tokenise},
    typ::{Implementation, Program, TypeExpr, type_id},
    value::{Value, primitive::IO},
    vm::{RunResult, evaluate},
};
use ron::ser::PrettyConfig;

fn main() {
    if let Some(arg1) = env::args().collect::<Vec<_>>().get(1) {
        if arg1 == "run" {
            let (mut module, runresult): (Program, RunResult) =
                ron::de::from_bytes(&fs::read("main.pvc").unwrap()).unwrap();

            let mut vars: HashMap<usize, Option<Value>> = HashMap::new();

            vars.insert(0, Some(Value::new(IO, type_id::IO)));
            vars.insert(1, Some(Value::new(IO, type_id::IO)));

            run_entire_program(&mut module, runresult, &mut vars);
            return;
        }
    }

    let file = "main.pv";
    let src = String::from_utf8(fs::read(file).unwrap()).unwrap();
    let tokens = tokenise(&src, 0, Cow::Owned(file.to_owned()));
    match tokens {
        Err(err) => {
            let (line, column) = get_line_and_column(&src, err.idx.index).unwrap();
            eprintln!("{:?} at {}:{line}:{column}", err.error, err.idx.file);
        }
        Ok(tokens) => {
            let module = parse_module(&tokens);
            match module {
                Ok(mut module) => {
                    fs::write("ir.ir", format!("{module:#?}")).unwrap();

                    let main = module.get_template("main").cloned();

                    let eval = if let Some(InfoTypeExpr {
                        idx: _,
                        expr: TypeExpr::Function(_name, _generics, Some(Implementation::Normal(f))),
                    }) = main
                    {
                        let cio = Some(Value::new(IO, type_id::IO));
                        let mut args = HashMap::from([(0, cio), (1, None)]);
                        evaluate(&mut module, f, &mut args, 0, vec![])
                    } else {
                        panic!("No main function")
                    };

                    let mut poisoned_vars = HashMap::new();
                    poisoned_vars.insert(0, Usage::Value);

                    let optimized = match eval {
                        RunResult::Residualise => unreachable!(),
                        RunResult::Concrete(c) => RunResult::Concrete(c),
                        RunResult::Partial(p) => RunResult::Partial(Partial {
                            blocks: remove_unused(&p.blocks, p.start_block, poisoned_vars),
                            start_block: p.start_block,
                            generics: p.generics,
                        }),
                    };

                    fs::write("eval.ir", format!("{optimized:#?}")).unwrap();

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

                    vars.insert(0, Some(Value::new(IO {}, type_id::IO)));
                    vars.insert(1, Some(Value::new(IO {}, type_id::IO)));

                    run_entire_program(&mut module, optimized, &mut vars);
                }
                Err(err) => {
                    let (line, column) = get_line_and_column(&src, err.span.index).unwrap();
                    eprintln!(
                        "ParseError: {:?} at {}:{line}:{column}",
                        err.data, err.span.file
                    );
                }
            }
        }
    }
}

fn run_entire_program(
    module: &mut Program,
    eval: RunResult,
    vars: &mut HashMap<usize, Option<Value>>,
) -> bool {
    match eval {
        RunResult::Concrete(_) => false,
        RunResult::Partial(p) => {
            // vars.insert(1, Some(Box::new(IO {})));
            let e = evaluate(module, p.blocks, vars, p.start_block, p.generics);
            run_entire_program(module, e, vars)
        }
        RunResult::Residualise => panic!(),
    }
}
