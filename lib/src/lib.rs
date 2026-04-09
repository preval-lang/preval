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

pub mod ir;
pub mod optimizations;
pub mod parser;
pub mod tokeniser;
pub mod value;
pub mod vm;
