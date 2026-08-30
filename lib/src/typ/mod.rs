use std::{borrow::Cow, collections::HashMap};

mod error;
pub use error::*;
use serde::{Deserialize, Serialize};

use crate::{
	error::Span,
	ir::{Block, IRContext, Terminal, to_ir},
	parser::{expression::InfoExpr, module::Module, typ::InfoTypeExpr},
	passes::type_check_expr::{Scope, infer_expr_type},
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
	Module(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Type {
	Concrete(ConcreteType),
	Union(usize, usize),
	EarlyReturn,
	Placeholder(usize),
	UnificationVar(usize),
}

#[derive(Debug, Clone)]
pub enum TypeExpr<'a> {
	Union(Box<InfoTypeExpr<'a>>, Box<InfoTypeExpr<'a>>),
	Name(String, Vec<Option<InfoTypeExpr<'a>>>),
	Subtype(
		Option<Box<InfoTypeExpr<'a>>>,
		String,
		Vec<Option<InfoTypeExpr<'a>>>,
	),
	Parameter(usize),
	Struct(HashMap<String, InfoTypeExpr<'a>>),
	Tuple(Vec<InfoTypeExpr<'a>>),
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
		Vec<InfoTypeExpr<'a>>,
		Box<InfoTypeExpr<'a>>,
		GenericImplementation<'a>,
		Vec<String>,
	),

	Module(Module<'a>),

	Trait(Trait<'a>),
}

#[derive(Debug, Clone)]
pub enum TraitMemberExpr<'a> {
	Function {
		generics: Vec<String>,
		args: Vec<InfoTypeExpr<'a>>,
		returns: InfoTypeExpr<'a>,
	},
	AssociatedType,
}

#[derive(Debug, Clone)]
pub struct Trait<'a> {
	pub members: HashMap<String, TraitMemberExpr<'a>>,
	pub impls: HashMap<String, TraitImplExpr<'a>>,
}

#[derive(Debug, Clone)]
pub struct TraitImplExpr<'a> {
	pub on: InfoTypeExpr<'a>,
	pub members: HashMap<String, InfoTypeExpr<'a>>,
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
pub struct Template<'a> {
	pub parameters: usize,
	pub expr: InfoTypeExpr<'a>,
}

#[derive(Debug, Clone)]
pub struct Instantiator<'a> {
	pub global_namespace: Module<'a>,
	pub types: Vec<Type>,
	next_unification_var: usize,
	pub subtype_members: HashMap<usize, Module<'a>>,
	ins_builtins: bool,
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

impl<'a> Instantiator<'a> {
	pub fn new() -> Self {
		let mut global_namespace = Module {
			methods: HashMap::new(),
			path: Vec::new(),
			templates: HashMap::new(),
			impls: Vec::new(),
		};

		for (name, typ) in TYPE_NAMES.iter().zip(TYPES) {
			global_namespace.templates.insert(
				name.to_string(),
				Template {
					parameters: 0,
					expr: InfoTypeExpr {
						expr: typ.clone(),
						idx: Span {
							file: Cow::Borrowed(file!().into()),
							index: 0,
						},
					},
				},
			);
		}

		let this = Instantiator {
			global_namespace,
			types: Vec::new(),
			next_unification_var: 0,
			ins_builtins: false,
			subtype_members: HashMap::new(),
		};

		this
	}

	pub fn instantiate_name(
		&mut self,
		n: &String,
		params: &Vec<Option<InfoTypeExpr<'a>>>,
		span: &Span<'a>,
		generics: &[usize],
		namespace: Option<usize>,
	) -> Result<usize, InfoTypeError<'a>> {
		let namespace = if let Some(namespace) = namespace {
			match self.subtype_members.get(&namespace) {
				Some(namespace) => namespace,
				None => {
					return Err(InfoTypeError {
						span: span.clone(),
						error: TypeError::NotAParent,
					});
				}
			}
		} else {
			&self.global_namespace
		};

		let methods = namespace.methods.get(n).cloned();
		let path = namespace.path.clone();

		let template = match namespace.templates.get(n) {
			Some(temp) => temp.clone(),
			None => {
				return Err(InfoTypeError {
					span: span.clone(),
					error: TypeError::UnknownType(n.clone()),
				});
			}
		};

		if params.len() > template.parameters {
			return Err(InfoTypeError {
				span: span.clone(),
				error: TypeError::IncorrectArgumentCount {
					expected: template.parameters,
					got: params.len(),
				},
			});
		}

		let mut template_params = Vec::new();

		for param in params {
			template_params.push(if let Some(param) = param {
				self.instantiate(param, generics)?
			} else {
				self.unification_var()
			});
		}

		while template_params.len() < template.parameters {
			template_params.push(self.unification_var());
		}

		let tid = self.instantiate(&template.expr, &template_params)?;

		if let Some(methods) = methods {
			if !self.subtype_members.contains_key(&tid) {
				self.subtype_members.insert(
					tid,
					Module {
						templates: HashMap::new(),
						methods: HashMap::new(),
						impls: Vec::new(),
						path,
					},
				);
			}
			let subtype_members = self.subtype_members.get_mut(&tid).unwrap();
			for (name, template) in methods {
				subtype_members.templates.insert(name, template);
			}
		}

		Ok(tid)
	}

	pub fn unification_var(&mut self) -> usize {
		self.next_unification_var += 1;
		self.add(Type::UnificationVar(self.next_unification_var - 1))
	}

	pub fn instantiate(
		&mut self,
		expr: &InfoTypeExpr<'a>,
		generics: &[usize],
	) -> Result<usize, InfoTypeError<'a>> {
		if !self.ins_builtins {
			for name in TYPE_NAMES {
				self.ins_builtins = true;
				self.instantiate(
					&InfoTypeExpr {
						expr: TypeExpr::Name(name.to_string(), vec![]),
						idx: Span {
							file: Cow::Borrowed(file!().into()),
							index: 0,
						},
					},
					&vec![],
				)
				.unwrap();
			}
		}

		let type_ = match &expr.expr {
			TypeExpr::Trait(_) => {
				todo!("traits")
			}
			TypeExpr::Module(module) => {
				let typ = self.add(Type::Concrete(ConcreteType::Module(module.path.clone())));
				self.subtype_members.insert(typ, module.clone());
				typ
			}
			TypeExpr::Subtype(parent, child_name, child_generics) => match parent {
				None => {
					self.instantiate_name(child_name, child_generics, &expr.idx, generics, None)?
				}
				Some(t) => {
					let typ = self.instantiate(t, generics)?;
					self.instantiate_name(child_name, child_generics, &t.idx, generics, Some(typ))?
				}
			},
			TypeExpr::Parameter(i) => generics[*i],
			TypeExpr::Name(n, params) => {
				self.instantiate_name(n, params, &expr.idx, generics, None)?
			}
			TypeExpr::Union(a, b) => {
				let a = self.instantiate(a.as_ref(), generics)?;
				let b = self.instantiate(&b, generics)?;
				self.add(Type::Union(a, b))
			}
			TypeExpr::Struct(fields) => {
				let mut ins_fields = HashMap::new();

				for field in fields {
					ins_fields.insert(field.0.clone(), self.instantiate(field.1, generics)?);
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
					ins_elems.push(self.instantiate(elem, generics)?);
				}
				self.add(Type::Concrete(ConcreteType::Tuple(ins_elems)))
			}
			TypeExpr::Function(args, ret_e, imp, arg_names) => {
				let mut ins_args = Vec::new();

				let mut scope = Scope::new();

				for (arg, name) in args.iter().zip(arg_names) {
					let t = self.instantiate(arg, generics)?;
					ins_args.push(t);
					scope.insert(name.clone(), t);
				}
				let ret = self.instantiate(ret_e, generics)?;

				let imp = match imp {
					GenericImplementation::Native(native) => Implementation::Native(native.clone()),
					GenericImplementation::Normal(body) => {
						Implementation::Normal(self.lower_function_body(
							body, &mut scope, ret, &ret_e.idx, arg_names, generics,
						)?)
					}
				};

				self.add(Type::Concrete(ConcreteType::Function(ins_args, ret, imp)))
			}
		};

		Ok(type_)
	}

	pub fn add(&mut self, typ: Type) -> usize {
		if let Some((id, _)) = self
			.types
			.iter()
			.enumerate()
			.find(|(_, old_typ)| typ == **old_typ)
		{
			id
		} else {
			self.types.push(typ);
			self.types.len() - 1
		}
	}

	pub fn get_type(&self, index: usize) -> Option<&Type> {
		self.types.get(index)
	}

	/// Looks up and clones the `Type` at `index`. Every type id handed out by
	/// this `Instantiator` is expected to stay valid for its lifetime, so an
	/// invalid index here means an internal invariant was broken elsewhere.
	pub fn type_of(&self, index: usize) -> Type {
		self.get_type(index)
			.cloned()
			.expect("internal type index should always be valid")
	}

	/// Returns the field map if `idx` names a struct type, otherwise the
	/// mismatched `Type` so the caller can build a `NotAStruct` error.
	pub fn expect_struct(&self, idx: usize) -> Result<HashMap<String, usize>, Type> {
		match self.type_of(idx) {
			Type::Concrete(ConcreteType::Struct(fields)) => Ok(fields),
			other => Err(other),
		}
	}

	/// Returns the (argument types, return type) if `idx` names a function
	/// type, otherwise the mismatched `Type` so the caller can build a
	/// `NotAFunction` error.
	pub fn expect_function(&self, idx: usize) -> Result<(Vec<usize>, usize), Type> {
		match self.type_of(idx) {
			Type::Concrete(ConcreteType::Function(args, ret, _imp)) => Ok((args, ret)),
			other => Err(other),
		}
	}

	/// Checks whether a value of type `assignee` may be used somewhere `slot`
	/// is expected. If `slot` is still an unresolved unification variable,
	/// this also *binds* it to `assignee`'s type as a side effect - so despite
	/// the name, this is a combined compatibility check + unification step,
	/// not a pure predicate.
	pub fn compatible(&mut self, assignee: usize, slot: usize) -> bool {
		let assignee_t = self.type_of(assignee);
		if let Type::EarlyReturn = assignee_t {
			return true;
		}
		let slot_t = self.type_of(slot);

		match &slot_t {
			Type::Concrete(a) => match &assignee_t {
				Type::Concrete(b) => a == b,
				Type::Union(a, b) => self.compatible(*a, slot) || self.compatible(*b, slot),
				Type::Placeholder(_) => slot_t == assignee_t,
				Type::UnificationVar(_) => {
					self.types[slot] = assignee_t;
					true
				}
				Type::EarlyReturn => panic!("Early return can't be assigned"),
			},
			Type::Union(a, b) => self.compatible(assignee, *a) || self.compatible(assignee, *b),
			Type::EarlyReturn => panic!("Early return can't be a slot"),
			Type::Placeholder(_) => slot_t == assignee_t,
			Type::UnificationVar(_) => {
				self.types[slot] = assignee_t;
				true
			}
		}
	}

	/// Type-checks a function body against its declared return type and
	/// lowers it to IR blocks. Kept separate from `instantiate` so that
	/// turning a `TypeExpr::Function` into a `ConcreteType::Function` (a
	/// type-level operation) stays distinct from type-checking and
	/// compiling its body (an expression/IR-level operation).
	pub fn lower_function_body<'b>(
		&mut self,
		body: &InfoExpr<'a>,
		scope: &mut Scope,
		ret: usize,
		ret_span: &Span<'a>,
		arg_names: &[String],
		generics: &[usize],
	) -> Result<Vec<Block>, InfoTypeError<'a>> {
		let body_type = infer_expr_type(body.clone(), self, scope, ret, generics)?;

		if !self.compatible(body_type.typ, ret) {
			return Err(InfoTypeError {
				span: ret_span.clone(),
				error: TypeError::IncompatibleTypes {
					expected: self.type_of(ret),
					got: self.type_of(body_type.typ),
				},
			});
		}

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
		};

		let mut block = 0;
		to_ir(&mut block, body_type, Some(last_var2), true, &mut context);

		Ok(ir)
	}
}
