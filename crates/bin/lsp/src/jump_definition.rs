use std::{collections::HashMap, ops::Deref};

use im_rc::Vector;
use lmt_parser::{Expr, Function, Spanned, Stmt};

/// return (need_to_continue_search, founded reference)
pub fn get_definition<'src>(
    ast: &'src HashMap<&'src str, Function>,
    ident_offset: usize,
) -> Option<Spanned<&'src str>> {
    let mut vector = Vector::new();
    for (_, function) in ast.iter() {
        let span = function.name.span();

        if span.start < ident_offset && span.end > ident_offset {
            return Some(function.name.clone());
        }
        if span.end < ident_offset {
            vector.push_back(function.name.clone());
        }
    }

    for (_, function) in ast.iter() {
        let args = function.args.iter().cloned().collect::<Vector<_>>();
        // if let (_, Some(value)) =
        //     get_definition_of_expr(&v.body, args + vector.clone(), ident_offset)
        // {
        //     return Some(value);
        // }

        let mut value = None;
        for stmt in &function.body {
            if let (_, Some(v)) =
                get_definition_of_stmt(stmt, &args + &vector.clone(), ident_offset)
            {
                value = Some(v);
            }
        }

        if let Some(value) = value {
            return Some(value);
        }
    }
    None
}

pub fn get_definition_of_stmt<'src>(
    stmt: &Spanned<Stmt>,
    definitions: Vector<Spanned<&'src str>>,
    ident_offset: usize,
) -> (bool, Option<Spanned<&'src str>>) {
    match stmt.deref() {
        Stmt::Error => todo!(),
        Stmt::Expr(expr) => get_definition_of_expr(expr, definitions, ident_offset),
        Stmt::Let(_ident, value) => {
            // let new_decl = Vector::unit((ident.deref(), ident.span()));

            match get_definition_of_expr(value, definitions.clone(), ident_offset) {
                (_, None) => (false, None),
                (_, Some(value)) => (false, Some(value)),
            }
        }
    }
}

pub fn get_definition_of_expr<'src>(
    expr: &Spanned<Expr>,
    definition_ass_list: Vector<Spanned<&'src str>>,
    ident_offset: usize,
) -> (bool, Option<Spanned<&'src str>>) {
    match expr.deref() {
        Expr::Error => (true, None),
        Expr::Literal(_) => (true, None),
        // Expr::List(exprs) => exprs
        //     .iter()
        //     .for_each(|expr| get_definition(expr, definition_ass_list)),
        Expr::Ident(ident) => {
            let span = ident.span();
            if ident_offset >= span.start && ident_offset < span.end {
                let index = definition_ass_list
                    .iter()
                    .position(|decl| decl.deref() == ident.deref());
                (
                    false,
                    index.map(|i| definition_ass_list.get(i).unwrap().clone()),
                )
            } else {
                (true, None)
            }
        }
        // Expr::Then(first, second) => {
        //     match get_definition_of_expr(first, definition_ass_list.clone(), ident_offset) {
        //         (true, None) => get_definition_of_expr(second, definition_ass_list, ident_offset),
        //         (false, None) => (false, None),
        //         (true, Some(value)) | (false, Some(value)) => (false, Some(value)),
        //     }
        // }
        Expr::Binary(lhs, _, rhs) => {
            match get_definition_of_expr(lhs, definition_ass_list.clone(), ident_offset) {
                (true, None) => get_definition_of_expr(rhs, definition_ass_list, ident_offset),
                (false, None) => (false, None),
                (true, Some(value)) | (false, Some(value)) => (false, Some(value)),
            }
        }
        Expr::Call(callee, args) => {
            match get_definition_of_expr(callee, definition_ass_list.clone(), ident_offset) {
                (true, None) => {}
                (true, Some(value)) => return (false, Some(value)),
                (false, None) => return (false, None),
                (false, Some(value)) => return (false, Some(value)),
            }
            for expr in args.deref() {
                match get_definition_of_expr(expr, definition_ass_list.clone(), ident_offset) {
                    (true, None) => continue,
                    (true, Some(value)) => return (false, Some(value)),
                    (false, None) => return (false, None),
                    (false, Some(value)) => return (false, Some(value)),
                }
            }
            (true, None)
        }
        Expr::If(test, consequent, alternative) => {
            match get_definition_of_expr(test, definition_ass_list.clone(), ident_offset) {
                (true, None) => {}
                (true, Some(value)) => return (false, Some(value)),
                (false, None) => return (false, None),
                (false, Some(value)) => return (false, Some(value)),
            }
            match get_definition_of_expr(consequent, definition_ass_list.clone(), ident_offset) {
                (true, None) => {}
                (true, Some(value)) => return (false, Some(value)),
                (false, None) => return (false, None),
                (false, Some(value)) => return (false, Some(value)),
            }
            match get_definition_of_expr(alternative, definition_ass_list, ident_offset) {
                (true, None) => (true, None),
                (true, Some(value)) => (false, Some(value)),
                (false, None) => (false, None),
                (false, Some(value)) => (false, Some(value)),
            }
        }
        Expr::Print(expr) => get_definition_of_expr(expr, definition_ass_list, ident_offset),
        Expr::List(lst) => {
            for expr in lst {
                match get_definition_of_expr(expr, definition_ass_list.clone(), ident_offset) {
                    (true, None) => continue,
                    (true, Some(value)) => return (false, Some(value)),
                    (false, None) => return (false, None),
                    (false, Some(value)) => return (false, Some(value)),
                }
            }
            (true, None)
        }
    }
}
