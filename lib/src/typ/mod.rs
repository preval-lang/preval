use std::{borrow::Cow, collections::HashMap};

mod error;
pub use error::*;
use serde::{Deserialize, Serialize};

use crate::{
	error::Span,
	ir::{Block, IRContext, Terminal, to_ir},
	parser::{expression::InfoExpr, typ::InfoTypeExpr},
	value::native::NativeFunction,
};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum IntegerSize {
	Size,
	Number(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConcreteType {
	Integer { size: IntegerSize, signed: bool },
	Float { size: usize },
	Bool,
	String,
	Struct(HashMap<String, usize>),
	Function(Vec<usize>, usize, Implementation),
	Tuple(Vec<usize>),
	IO,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Type {
	Concrete(ConcreteType),
	Union(usize, usize),
	EarlyReturn,
	Placeholder(usize),
}

pub type TypeExpr<'a> = BaseTypeExpr<'a, InfoTypeExpr<'a>>;

#[derive(Debug, Clone)]
pub enum BaseTypeExpr<'a, T> {
	Union(Box<T>, Box<T>),
	Name(Vec<String>, bool),
	Generics(Box<T>, Vec<T>),
	Parameter(usize),
	Struct(HashMap<String, T>),
	Tuple(Vec<T>),

	Integer {
		size: IntegerSize,
		signed: bool,
	},
	Float {
		size: usize,
	},
	Bool,
	String,
	IO,

	Function(
		Vec<T>,
		Box<T>,
		Option<GenericImplementation<'a>>,
		Vec<String>,
	),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Implementation {
	Native(NativeFunction),
	Normal(Vec<Block>),
}

#[derive(Debug, Clone)]
pub enum GenericImplementation<'a> {
	Native(NativeFunction),
	Normal(Box<InfoExpr<'a>>),
}

#[derive(Debug, Clone)]
pub struct Program<'a> {
	template_names: HashMap<Vec<String>, (Vec<String>, InfoTypeExpr<'a>)>,
	pub types: Vec<Type>,
}

macro_rules! type_ids {
    ($($name:ident => $expr:expr),* $(,)?) => {
        pub mod type_id {
            type_ids!(@consts [] ; $($name => $expr),*);
        }

        const TYPES: &[TypeExpr] = &[
            $($expr),*
        ];

        const TYPE_NAMES: &[&str] = &[
            $(stringify!($name)),*
        ];
    };

    (@consts [$($seen:ident),*] ; $head:ident => $expr:expr $(, $tail:ident => $tail_expr:expr)*) => {
        #[allow(non_upper_case_globals)]
        #[allow(dead_code)]
        pub const $head: usize = type_ids!(@count $($seen),*);
        type_ids!(@consts [$($seen,)* $head] ; $($tail => $tail_expr),*);
    };

    (@consts [$($seen:ident),*] ;) => {};

    (@count) => { 0 };
    (@count $($idents:ident),*) => {
        <[()]>::len(&[$(type_ids!(@replace $idents)),*])
    };

    (@replace $_t:ident) => { () };
}

type_ids! {
	usize => TypeExpr::Integer { size: IntegerSize::Size, signed: false },
	bool => TypeExpr::Bool,
	empty_tuple => TypeExpr::Tuple(vec![]),
	String => TypeExpr::String,
	IO => TypeExpr::IO,
}

impl<'a> Program<'a> {
	pub fn new() -> Self {
		let template_names = HashMap::new();
		let types = Vec::new();

		let mut this = Self {
			types,
			template_names,
		};

		for (name, typ) in TYPE_NAMES.iter().zip(TYPES) {
			this.add_template(
				vec![name.to_string()],
				InfoTypeExpr {
					expr: typ.clone(),
					idx: Span {
						file: Cow::Borrowed(file!().into()),
						index: 0,
					},
				},
			);

			this.instantiate(
				&InfoTypeExpr {
					expr: TypeExpr::Name(vec![name.to_string()], true),
					idx: Span {
						file: Cow::Borrowed(file!().into()),
						index: 0,
					},
				},
				&vec![],
				&vec![],
			)
			.unwrap();
		}

		this
	}

	pub fn instantiate(
		&mut self,
		expr: &InfoTypeExpr<'a>,
		generics: &[usize],
		prefix: &[String],
	) -> Result<usize, InfoTypeError<'a>> {
		let type_ = match &expr.expr {
			TypeExpr::Parameter(i) => generics[*i],
			TypeExpr::Name(n, global) => {
				let mut fqn = if *global { vec![] } else { prefix.to_vec() };
				fqn.extend_from_slice(n);
				let (declaring_prefix, template) = match self.template_names.get(&fqn) {
					Some(temp) => temp.clone(),
					None => {
						return Err(InfoTypeError {
							span: expr.idx.clone(),
							error: TypeError::UnknownType(n.clone()),
						});
					}
				};
				self.instantiate(&template, generics, &declaring_prefix)?
			}
			TypeExpr::Union(a, b) => {
				let a = self.instantiate(a.as_ref(), generics, prefix)?;
				let b = self.instantiate(&b, generics, prefix)?;
				self.add(Type::Union(a, b))
			}
			TypeExpr::Generics(base, params) => {
				let mut ins_params = Vec::new();

				for param in params {
					ins_params.push(self.instantiate(param, generics, prefix)?);
				}

				self.instantiate(base.as_ref(), &ins_params, prefix)?
			}
			TypeExpr::Struct(fields) => {
				let mut ins_fields = HashMap::new();

				for field in fields {
					ins_fields.insert(
						field.0.clone(),
						self.instantiate(field.1, generics, prefix)?,
					);
				}
				self.add(Type::Concrete(ConcreteType::Struct(ins_fields)))
			}
			TypeExpr::Bool => self.add(Type::Concrete(ConcreteType::Bool)),
			TypeExpr::String => self.add(Type::Concrete(ConcreteType::String)),
			TypeExpr::IO => self.add(Type::Concrete(ConcreteType::IO)),
			TypeExpr::Integer { size, signed } => self.add(Type::Concrete(ConcreteType::Integer {
				size: *size,
				signed: *signed,
			})),
			TypeExpr::Float { size } => {
				self.add(Type::Concrete(ConcreteType::Float { size: *size }))
			}
			TypeExpr::Tuple(elems) => {
				let mut ins_elems = Vec::new();

				for elem in elems {
					ins_elems.push(self.instantiate(elem, generics, prefix)?);
				}
				self.add(Type::Concrete(ConcreteType::Tuple(ins_elems)))
			}
			TypeExpr::Function(args, ret, imp, arg_names) => {
				let mut ins_args = Vec::new();
				for arg in args {
					ins_args.push(self.instantiate(arg, generics, prefix)?);
				}
				let ret = self.instantiate(ret, generics, prefix)?;

				let imp = match imp
					.as_ref()
					.expect("should not be null after implementation pass")
				{
					GenericImplementation::Native(native) => Implementation::Native(native.clone()),
					GenericImplementation::Normal(body) => Implementation::Normal({
						let mut last_var = arg_names.len();
						let last_var2 = last_var;
						let mut locals = HashMap::new();
						let mut ir = vec![Block {
							terminal: Terminal::Return(last_var),
							statements: Vec::new(),
						}];
						for (idx, arg) in arg_names.iter().enumerate() {
							locals.insert(arg.clone(), idx);
						}

						let mut context = IRContext {
							blocks: &mut ir,
							generics,
							ins: self,
							locals: &mut locals,
							next_var: &mut last_var,
							prefix,
						};

						let mut block = 0;
						to_ir(
							&mut block,
							*(body).clone(),
							Some(last_var2),
							true,
							&mut context,
						)
						.expect("propagate IR Error as type-error");

						ir
					}),
				};

				self.add(Type::Concrete(ConcreteType::Function(ins_args, ret, imp)))
			}
		};

		Ok(type_)
	}

	pub fn add(&mut self, typ: Type) -> usize {
		self.types.push(typ);
		self.types.len() - 1
	}

	pub fn add_template(&mut self, name: Vec<String>, expr: InfoTypeExpr<'a>) {
		self.template_names
			.insert(name.clone(), (name[..name.len() - 1].to_vec(), expr));
	}

	pub fn get_template(&'a self, name: &Vec<String>) -> Option<&'a InfoTypeExpr<'a>> {
		self.template_names.get(name).map(|a| &a.1)
	}

	pub fn get_type(&self, index: usize) -> Option<&Type> {
		self.types.get(index)
	}

	pub fn compatible(&self, assignee: usize, slot: usize, index: usize) -> Result<bool, ()> {
		let assignee_t = self.get_type(assignee).ok_or(())?;
		if let Type::EarlyReturn = assignee_t {
			return Ok(true);
		}
		let slot_t = self.get_type(slot).ok_or(())?;

		match slot_t {
			Type::Concrete(a) => match assignee_t {
				Type::Concrete(b) => Ok(a == b),
				Type::Union(a, b) => Ok(self.compatible(*a, slot, index + 1)?
					|| self.compatible(*b, slot, index + 1)?),
				Type::Placeholder(_) => Ok(slot_t == assignee_t),
				Type::EarlyReturn => panic!("Early return can't be assigned"),
			},
			Type::Union(a, b) => Ok(self.compatible(assignee, *a, index + 1)?
				|| self.compatible(assignee, *b, index + 1)?),
			Type::EarlyReturn => panic!("Early return can't be a slot"),
			Type::Placeholder(_) => Ok(slot_t == assignee_t),
		}
	}
}
