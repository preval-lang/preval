use std::{collections::HashMap, env, fs};

use ron::ser::PrettyConfig;

use crate::{
    ir::{Module, module_to_string},
    parser::module::parse_module,
    passes::remove_unused::remove_unused,
    tokeniser::{get_line_and_column, tokenise},
    value::{Value, primitive::IO},
    vm::{RunResult, evaluate},
};

pub mod ir;
pub mod parser;
pub mod passes;
pub mod tokeniser;
pub mod value;
pub mod vm;
