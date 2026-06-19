use std::{borrow::Cow, collections::HashMap, fs::read_dir, path::PathBuf};

use preval_lib::{
	error::Span,
	ir::Partial,
	parser::{module::declaration_pass, typ::InfoTypeExpr},
	passes::remove_unused::{Usage, remove_unused},
	tokeniser::{get_line_and_column, tokenise},
	typ::{ConcreteType, Implementation, Instantiator, Template, Type, TypeExpr, type_id},
	value::{Value, primitive::IO},
	vm::{RunResult, evaluate},
};
use ron::ser::PrettyConfig;

fn add_dir(path: PathBuf, module: &mut HashMap<String, Template<'_>>, path_strings: Vec<String>) {
	for entry in read_dir(path).unwrap() {
		let entry = entry.unwrap();
		if entry.file_type().unwrap().is_dir() {
			let mut child = HashMap::new();
			let mut new_path_strings = path_strings.clone();
			new_path_strings.push(entry.file_name().into_string().unwrap());
			add_dir(entry.path(), &mut child, new_path_strings.clone());
			if let Some(_) = module.insert(
				entry.file_name().into_string().unwrap(),
				Template {
					parameters: 0,
					expr: InfoTypeExpr {
						expr: TypeExpr::Module(child, new_path_strings),
						idx: Span {
							file: Cow::Owned("asldjlasjd".to_owned()),
							index: 0,
						},
					},
				},
			) {
				panic!("duplicate modules {path_strings:?}");
			}
		} else if entry.file_type().unwrap().is_file() {
			if entry.path().extension().unwrap() == "pv" {
				let contents = std::fs::read_to_string(entry.path()).unwrap();

				let tokens = match tokenise(
					&contents,
					0,
					Cow::Owned(entry.path().to_str().unwrap().to_owned()),
				) {
					Ok(tokens) => tokens,
					Err(error) => {
						let (line, column) =
							get_line_and_column(&contents, error.idx.index).unwrap();
						panic!(
							"{:?} at {:?}:{line}:{column}",
							error.error,
							entry.path().to_str().unwrap()
						);
					}
				};

				match declaration_pass(&tokens, module) {
					Ok(tokens) => tokens,
					Err(error) => {
						let (line, column) =
							get_line_and_column(&contents, error.span.index).unwrap();
						panic!(
							"{:?} at {:?}:{line}:{column}",
							error.data,
							entry.path().to_str().unwrap()
						);
					}
				};
			}
		}
	}
}

fn compile(project_path: Vec<PathBuf>) -> (RunResult, Vec<Type>) {
	println!("TODO: Solve unification types");
	println!("TODO: Check types 💀");

	let mut ins = Instantiator::new();

	for path in project_path {
		add_dir(path, &mut ins.global_namespace, vec![]);
	}

	// match implementation_pass(symbols, &mut module) {
	// 	Ok(tokens) => tokens,
	// 	Err(error) => {
	// 		let (line, column) = get_line_and_column(
	// 			&std::fs::read_to_string(error.span.file.as_ref()).unwrap(),
	// 			error.span.index,
	// 		)
	// 		.unwrap();
	// 		panic!("{:?} at {:?}:{line}:{column}", error.data, error.span.file);
	// 	}
	// };

	let main_type_id = ins
		.instantiate(
			&InfoTypeExpr {
				expr: TypeExpr::Name("main".to_string(), vec![]),
				idx: Span {
					file: Cow::Borrowed(file!().into()),
					index: 0,
				},
			},
			&vec![],
		)
		.unwrap();

	let mut types = ins.types;

	let eval = if let Type::Concrete(ConcreteType::Function(_, _, Implementation::Normal(imp))) =
		types[main_type_id].clone()
	{
		let cio = Some(Value::new(IO, type_id::IO));
		let mut args = HashMap::from([(0, cio), (1, None)]);
		evaluate(&mut types, imp.clone(), &mut args, 0, vec![])
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

	(optimized, types)
}

fn run(main: RunResult, mut types: Vec<Type>) {
	let mut vars: HashMap<usize, Option<Value>> = HashMap::new();

	vars.insert(0, Some(Value::new(IO {}, type_id::IO)));
	vars.insert(1, Some(Value::new(IO {}, type_id::IO)));

	run_entire_program(&mut types, main, &mut vars);
}

fn main() {
	let mut args = std::env::args();

	args.next();

	match args.next().as_ref().map(|f| f.as_str()) {
		Some("compile") => {
			let mut project_paths = Vec::new();
			for path in args {
				project_paths.push(PathBuf::from(&path));
			}

			let vec = ron::ser::to_string_pretty(&compile(project_paths), PrettyConfig::default())
				.unwrap();
			std::fs::write("out.pvc", vec).unwrap();
		}
		Some("run") => {
			let bin = std::fs::read_to_string(args.next().unwrap()).unwrap();
			let (main, types): (RunResult, Vec<Type>) = ron::from_str(&bin).unwrap();
			run(main, types);
		}
		_ => {
			eprintln!("Subcommands:\n\tcompile [...module paths]\n\trun [.pvc file]")
		}
	}
}

fn run_entire_program(
	module: &mut Vec<Type>,
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
