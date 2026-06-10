use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use preval_lib::{
	error::Span,
	ir::Partial,
	parser::{
		module::{declaration_pass, implementation_pass},
		typ::InfoTypeExpr,
	},
	passes::remove_unused::{Usage, remove_unused},
	tokeniser::{get_line_and_column, tokenise},
	typ::{ConcreteType, Implementation, Program, Type, TypeExpr, type_id},
	value::{Value, primitive::IO},
	vm::{RunResult, evaluate},
};
use ron::ser::PrettyConfig;

fn compile(project_path: Vec<PathBuf>) -> (RunResult, Vec<Type>) {
	let mut files = Vec::new();

	for path in project_path {
		get_all_pv_files(&mut files, vec![], path.into());
	}

	let mut symbols = HashMap::new();

	let mut module = Program::new();

	for (file, prefix) in files {
		let contents = std::fs::read_to_string(&file).unwrap();

		let tokens = match tokenise(&contents, 0, Cow::Owned(file.to_str().unwrap().to_owned())) {
			Ok(tokens) => tokens,
			Err(error) => {
				let (line, column) = get_line_and_column(&contents, error.idx.index).unwrap();
				panic!("{:?} at {:?}:{line}:{column}", error.error, file);
			}
		};

		match declaration_pass(&tokens, &mut module, &prefix, &mut symbols) {
			Ok(tokens) => tokens,
			Err(error) => {
				let (line, column) = get_line_and_column(&contents, error.span.index).unwrap();
				panic!("{:?} at {:?}:{line}:{column}", error.data, file);
			}
		};
	}

	match implementation_pass(symbols, &mut module) {
		Ok(tokens) => tokens,
		Err(error) => {
			let (line, column) = get_line_and_column(
				&std::fs::read_to_string(error.span.file.as_ref()).unwrap(),
				error.span.index,
			)
			.unwrap();
			panic!("{:?} at {:?}:{line}:{column}", error.data, error.span.file);
		}
	};

	let main_type_id = module
		.instantiate(
			&InfoTypeExpr {
				expr: TypeExpr::Name(vec!["main".to_string()], false),
				idx: Span {
					file: Cow::Borrowed(file!().into()),
					index: 0,
				},
			},
			&vec![],
			&vec![],
		)
		.unwrap();

	let mut types = module.types;

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

	// let (main_type_id, types) = compile(project_paths);
	// run(main_type_id, types);
}

fn get_all_pv_files(files: &mut Vec<(PathBuf, Vec<String>)>, path: Vec<String>, base: PathBuf) {
	let mut dir = base.clone();
	for subdir in &path {
		dir = dir.join(subdir);
	}
	for file in std::fs::read_dir(dir).unwrap() {
		let file = file.unwrap();
		if file.path().is_dir() {
			let mut mod_path = path.clone();
			mod_path.push(file.file_name().into_string().unwrap());
			get_all_pv_files(files, mod_path, base.clone());
		} else {
			files.push((file.path(), path.clone()));
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
