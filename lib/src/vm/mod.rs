mod operation;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
	ir::{Block, Callable, Function, Operation, Partial, Statement, Terminal},
	typ::{ConcreteType, Implementation, Program, Type},
	value::{Value, structure::Struct},
	vm::operation::{access, call, guard_phi, index, initialize_struct, is, load_local, phi},
};

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunResult {
	Concrete(Value),
	Partial(Partial),
	Residualise, // Native functions only! Because all preval functions can be partially evaluated even if there are no known arguments
}

pub fn evaluate(
	module: &mut Program,
	mut blocks: Vec<Block>,
	vars: &mut HashMap<usize, Option<Value>>,
	start_block: usize,
	mut generics: Vec<usize>,
) -> RunResult {
	let mut last_block_num = start_block;
	let mut block_num = start_block;

	loop {
		let mut out: Vec<Statement> = Vec::new();

		let old_vars: Vec<_> = vars.keys().cloned().collect();

		for stmt in blocks[block_num].statements.clone() {
			match stmt {
				Statement {
					store,
					operation: Operation::Is { value, typ },
				} => is(value, typ, module, vars, &mut out, store, &generics),
				Statement {
					store,
					operation: Operation::GuardPhi { block, var },
				} => guard_phi(block, var, store, last_block_num, &mut out, vars),
				Statement {
					store,
					operation: Operation::Call { function, args },
				} => call(function, args, store, &mut out, module, vars),
				Statement {
					store,
					operation: Operation::LoadFunction(type_expr),
				} => {
					let type_id = module
						.instantiate_rt(&type_expr, &generics)
						.expect("move this to compile time by specialising function body");
					let typ = module.get_type(type_id);
					if let Some(typ) = typ {
						if let Some(store) = store {
							vars.insert(
								store,
								Some(match typ {
									Type::Concrete(ConcreteType::Function(_, _, imp, generics)) => {
										match imp
											.clone()
											.expect("Function only declared and not implemented")
										{
											Implementation::Native(imp) => {
												Value::new(imp.clone(), type_id)
											}
											Implementation::Normal(imp) => Value::new(
												Function {
													ir: imp.clone(),
													exported: false,
													generics: generics.clone(),
												},
												type_id,
											),
										}
									}
									_ => todo!(),
								}),
							);
						}
					}
				}
				Statement {
					store,
					operation: Operation::LoadLiteral(value),
				} => {
					if let Some(store) = store {
						vars.insert(store, Some(value.clone()));
					}
				}
				Statement {
					store,
					operation: Operation::LoadLocal { src },
				} => {
					load_local(src, store, &mut out, vars);
				}
				Statement {
					store,
					operation: Operation::Index(left, right),
				} => {
					index(left, right, store, &mut out, vars);
				}
				Statement {
					store,
					operation: Operation::Phi { block_to_var },
				} => {
					phi(block_to_var, store, last_block_num, &mut out, vars);
				}
				Statement {
					store,
					operation: Operation::InitializeStruct(name, fields),
				} => {
					initialize_struct(name, fields, store, module, &mut out, vars, &generics);
				}
				Statement {
					store,
					operation: Operation::Access(left, right),
				} => {
					access(left, right, store, &mut out, vars);
				}
			}
		}

		let new_vars: Vec<_> = vars.keys().filter(|k| !old_vars.contains(k)).collect();

		let residualise = out.len() != 0;

		for var_num in new_vars {
			if let Some(Some(var)) = vars.get(var_num) {
				if let Some(struc) = var.data.as_any().downcast_ref::<Struct>() {
					let mut complete = true;
					for field in &struc.fields {
						if field.1.is_none() {
							complete = false;
							break;
						}
					}
					if complete {
						out.insert(
							0,
							Statement {
								store: Some(*var_num),
								operation: Operation::LoadLiteral(var.clone()),
							},
						);
					}
				} else {
					out.insert(
						0,
						Statement {
							store: Some(*var_num),
							operation: Operation::LoadLiteral(var.clone()),
						},
					);
				}
			}
		}

		match blocks[block_num].terminal.clone() {
			Terminal::Guard {
				dependency,
				body,
				continuation,
			} => match vars.get(&dependency) {
				Some(Some(_)) => {
					blocks[block_num] = Block {
						statements: out,
						terminal: Terminal::Jump(body),
					};
					last_block_num = block_num;
					block_num = body;
				}
				Some(None) => {
					blocks[block_num] = Block {
						statements: out,
						terminal: Terminal::Jump(body),
					};
					last_block_num = block_num;
					block_num = continuation;
				}
				None => panic!("undefined variable in guard"),
			},
			Terminal::TailCall { function, args } => {
				let mut callable_var = None;
				let ir: Option<Partial> = match function {
					Callable::Var(var) => {
						callable_var = Some(var);
						if let Some(value) = vars.get(&var) {
							if let Some(value) = value {
								if let Some(result) = value.data.as_any().downcast_ref::<Function>()
								{
									Some(Partial {
										blocks: result.ir.clone(),
										start_block: 0,
										generics: result.generics.clone(),
									})
								} else {
									match value
										.clone()
										.data
										.call(module, args.iter().map(|idx| &vars[idx]).collect())
									{
										RunResult::Concrete(return_value) => {
											if residualise {
												out.push(Statement {
													store: { Some(90000) },
													operation: Operation::LoadLiteral(return_value),
												});
												blocks[block_num] = Block {
													statements: out,
													terminal: Terminal::Return(90000),
												};
												return RunResult::Partial(Partial {
													blocks,
													start_block,
													generics: vec![],
												});
											} else {
												return RunResult::Concrete(return_value);
											}
										}
										RunResult::Partial(p) => Some(p),
										RunResult::Residualise => None,
									}
								}
							} else {
								None
							}
						} else {
							panic!("Undefined variable in tail call")
						}
					}
					Callable::Partial(partial) => Some(partial),
				};
				let new = if let Some(ir) = ir {
					ir
				} else {
					blocks[block_num] = Block {
						statements: out,
						terminal: Terminal::TailCall {
							function: Callable::Var(callable_var.unwrap()),
							args,
						},
					};

					return RunResult::Partial(Partial {
						blocks,
						start_block,
						generics: generics.to_vec(),
					});
				};

				let mut new_vars = HashMap::new();

				for (idx, arg) in args.iter().enumerate() {
					new_vars.insert(idx, vars.get(&arg).expect("Defined variable").clone());
				}

				*vars = new_vars;

				if residualise {
					match evaluate(module, new.blocks, vars, new.start_block, new.generics) {
						RunResult::Concrete(val) => {
							out.push(Statement {
								store: {
									println!("todo: use next var {}:{}", file!(), line!());
									Some(90000)
								},
								operation: Operation::LoadLiteral(val),
							});
							blocks[block_num] = Block {
								statements: out,
								terminal: Terminal::Return(90000),
							};
							return RunResult::Partial(Partial {
								blocks,
								start_block,
								generics: generics.to_vec(),
							});
						}
						RunResult::Partial(p) => {
							blocks[block_num] = Block {
								statements: out,
								terminal: Terminal::TailCall {
									function: Callable::Partial(p),
									args,
								},
							};
						}
						RunResult::Residualise => {
							blocks[block_num].statements = out;
							return RunResult::Partial(Partial {
								blocks,
								start_block,
								generics: generics.to_vec(),
							});
						}
					}

					return RunResult::Partial(Partial {
						blocks,
						start_block,
						generics: generics.to_vec(),
					});
				} else {
					blocks = new.blocks;
					last_block_num = block_num;
					block_num = new.start_block;
					generics = new.generics;
				}

				continue;
			}
			Terminal::CondJump { cond, then, els } => match vars.get(&cond) {
				Some(Some(value)) => {
					if let Some(cond_bool) = value.data.as_any().downcast_ref::<bool>() {
						let next_block = if *cond_bool { then } else { els };

						blocks[block_num] = Block {
							statements: out,
							terminal: Terminal::Jump(next_block),
						};
						last_block_num = block_num;
						block_num = next_block;
					} else {
						panic!("Non-bool condition")
					}
				}
				Some(None) => {
					blocks[block_num] = Block {
						statements: out,
						terminal: Terminal::Branch {
							cond: cond,
							then: evaluate(module, blocks.clone(), vars, then, generics.clone()),
							els: evaluate(module, blocks.clone(), vars, els, generics.clone()),
						},
					};
					return RunResult::Partial(Partial {
						blocks,
						start_block,
						generics,
					});
				}
				None => panic!("Undefined variable in condition"),
			},
			Terminal::Branch { cond, then, els } => match vars.get(&cond) {
				Some(Some(value)) => {
					if let Some(cond_bool) = value.data.as_any().downcast_ref::<bool>() {
						if *cond_bool {
							return then.clone();
						} else {
							return els.clone();
						}
					} else {
						panic!("Non-bool condition")
					}
				}
				Some(None) => {
					blocks[block_num] = Block {
						statements: out,
						terminal: Terminal::Branch {
							cond: cond,
							then: then.clone(),
							els: els.clone(),
						},
					};
					return RunResult::Partial(Partial {
						blocks,
						start_block,
						generics: generics.to_vec(),
					});
				}
				None => panic!("Undefined variable in condition"),
			},
			Terminal::Jump(dest) => {
				blocks[block_num] = Block {
					statements: out,
					terminal: Terminal::Jump(dest),
				};
				last_block_num = block_num;
				block_num = dest;
			}
			Terminal::Return(var) => {
				blocks[block_num] = Block {
					statements: out,
					terminal: Terminal::Return(var),
				};
				if !residualise {
					match vars.get(&var) {
						Some(Some(var)) => {
							return RunResult::Concrete(var.clone());
						}
						Some(None) => {
							return RunResult::Partial(Partial {
								blocks,
								start_block,
								generics: generics.to_vec(),
							});
						}
						None => panic!("Undefined variable in return"),
					}
				}
				return RunResult::Partial(Partial {
					blocks,
					start_block,
					generics: generics.to_vec(),
				});
			}
		}
	}
}
