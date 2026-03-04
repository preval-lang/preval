use std::collections::HashMap;

use crate::{
    expression_parser::{Expr, InfoExpr},
    tokeniser::Literal,
};

pub fn preval(expr: &InfoExpr, scopes: &mut Vec<&mut HashMap<String, Literal>>) -> InfoExpr {
    match expr.expr {
        Expr::Block(statements, returns) => {
            let scope = HashMap::new();
            scopes.push(&mut scope);
            InfoExpr {
                idx: expr.idx,
                expr: Expr::Block(
                    statements.iter().map(|stmt| preval(stmt, scopes)).collect(),
                    returns,
                ),
            }
        }
    }
}

pub fn get_name<'a>(
    name: &str,
    scopes: &'a Vec<&'a mut HashMap<String, Literal>>,
) -> Option<&'a Literal> {
    for scope in scopes.iter().rev() {
        if let Some(v) = scope.get(name) {
            return Some(v);
        }
    }
    None
}
