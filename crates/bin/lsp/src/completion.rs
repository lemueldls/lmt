use std::collections::HashMap;

use crate::parser::{Expr, Function, Spanned};
pub enum ImCompleteCompletionItem<'src> {
    Variable(&'src str),
    Function(String, Vec<&'src str>),
}
/// return (need_to_continue_search, founded reference)
pub fn completion<'src>(
    ast: &'src HashMap<&'src str, Function>,
    ident_offset: usize,
) -> HashMap<String, ImCompleteCompletionItem<'src>> {
    let mut map = HashMap::new();
    for (_, v) in ast.iter() {
        if v.name.1.end < ident_offset {
            map.insert(
                v.name.0.to_string(),
                ImCompleteCompletionItem::Function(
                    v.name.0.to_string(),
                    v.args.clone().into_iter().map(|name| name.0).collect(),
                ),
            );
        }
    }

    // collect params variable
    for (_, v) in ast.iter() {
        if v.span.end > ident_offset && v.span.start < ident_offset {
            // log::debug!("this is completion from body {}", name);
            v.args.iter().for_each(|(item, _)| {
                map.insert(item.to_string(), ImCompleteCompletionItem::Variable(item));
            });
            for expr in &v.body {
                get_completion_of(expr, &mut map, ident_offset);
            }
        }
    }
    map
}

pub fn get_completion_of<'src>(
    expr: &Spanned<Expr<'src>>,
    definition_map: &mut HashMap<String, ImCompleteCompletionItem<'src>>,
    ident_offset: usize,
) -> bool {
    match &expr.0 {
        Expr::Error => true,
        Expr::Value(_) => true,
        // Expr::List(exprs) => exprs
        //     .iter()
        //     .for_each(|expr| get_definition(expr, definition_ass_list)),
        Expr::Local(local) => !(ident_offset >= local.1.start && ident_offset < local.1.end),
        Expr::Let(typed_ident, value) => {
            let (name, name_span) = typed_ident.ident;
            definition_map.insert(name.to_string(), ImCompleteCompletionItem::Variable(name));
            // match get_completion_of(lhs, definition_map, ident_offset) {
            //     true => {
            //         if let Some(body) = body {
            //             get_completion_of(body, definition_map, ident_offset)
            //         } else {
            //             false
            //         }
            //     }
            //     false => false,
            // }

            false
        }
        Expr::Then(first, second) => {
            match get_completion_of(first, definition_map, ident_offset) {
                true => get_completion_of(second, definition_map, ident_offset),
                false => false,
            }
        }
        Expr::Binary(lhs, _op, rhs) => {
            match get_completion_of(lhs, definition_map, ident_offset) {
                true => get_completion_of(rhs, definition_map, ident_offset),
                false => false,
            }
        }
        Expr::Call(callee, args) => {
            match get_completion_of(callee, definition_map, ident_offset) {
                true => {}
                false => return false,
            }
            for expr in &args.0 {
                match get_completion_of(expr, definition_map, ident_offset) {
                    true => continue,
                    false => return false,
                }
            }
            true
        }
        // Expr::If(test, consequent, alternative) => {
        //     match get_completion_of(test, definition_map, ident_offset) {
        //         true => {}
        //         false => return false,
        //     }
        //     match get_completion_of(consequent, definition_map, ident_offset) {
        //         true => {}
        //         false => return false,
        //     }
        //     get_completion_of(alternative, definition_map, ident_offset)
        // }
        // Expr::Print(expr) => get_completion_of(expr, definition_map, ident_offset),
        Expr::List(lst) => {
            for expr in lst {
                match get_completion_of(expr, definition_map, ident_offset) {
                    true => continue,
                    false => return false,
                }
            }
            true
        }
    }
}
