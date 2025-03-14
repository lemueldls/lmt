use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
};

use chumsky::span::Span;
use im_rc::Vector;
use lmt_parser::{Expr, Function, Spanned};

#[derive(Debug, Clone)]
pub enum ReferenceSymbol<'src> {
    Founded(Spanned<&'src str>),
    Founding(usize),
}
use ReferenceSymbol::*;
pub fn get_reference<'src>(
    ast: &'src HashMap<&'src str, Function>,
    ident_offset: usize,
    include_self: bool,
) -> Vec<Spanned<&'src str>> {
    let mut vector = Vector::new();
    let mut reference_list = vec![];
    // for (_, v) in ast.iter() {
    //     if v.name.1.end < ident_offset {
    //         vector.push_back(v.name.clone());
    //     }
    // }
    let mut kv_list = ast.iter().collect::<Vec<_>>();
    kv_list.sort_by(|a, b| a.1.name.start().cmp(&b.1.name.start()));
    let mut reference_symbol = ReferenceSymbol::Founding(ident_offset);
    // let mut fn_vector = Vector::new();
    for (_, v) in kv_list {
        let (_, range) = &v.name;
        if ident_offset >= range.start && ident_offset < range.end {
            reference_symbol = ReferenceSymbol::Founded(v.name.clone());
            if include_self {
                reference_list.push(v.name.clone());
            }
        };
        vector.push_back(v.name.clone());
        let args = v
            .args
            .iter()
            .map(|arg| {
                if ident_offset >= arg.1.start && ident_offset < arg.1.end {
                    reference_symbol = ReferenceSymbol::Founded(arg.clone());
                    if include_self {
                        reference_list.push(arg.clone());
                    }
                }
                arg.clone()
            })
            .collect::<Vector<_>>();

        let definition_ass_list = args + vector.clone();
        let reference_symbol = reference_symbol.clone();
        for expr in &v.body {
            get_reference_of_expr(
                expr,
                &definition_ass_list,
                &reference_symbol,
                &mut reference_list,
                include_self,
            );
        }
    }
    reference_list
}

pub fn get_reference_of_expr<'src>(
    expr: &Spanned<Expr<'src>>,
    definition_ass_list: &Vector<Spanned<&'src str>>,
    reference_symbol: &ReferenceSymbol,
    reference_list: &mut Vec<Spanned<&'src str>>,
    include_self: bool,
) {
    match &expr.0 {
        Expr::Error => {}
        Expr::Value(_) => {}
        Expr::Local((name, span)) => {
            if let Founded((symbol_name, symbol_span)) = reference_symbol {
                if symbol_name == name {
                    let index = definition_ass_list
                        .iter()
                        .position(|decl| decl.0 == *symbol_name);
                    if let Some(symbol) = index.map(|i| definition_ass_list.get(i).unwrap()) {
                        if *symbol == (*symbol_name, *symbol_span) {
                            reference_list.push((name, span.clone()));
                        }
                    };
                }
            }
            // if ident_offset >= local.1.start && ident_offset < local.1.end {
            //     let index = definition_ass_list
            //         .iter()
            //         .position(|decl| decl.0 == local.0);
            //     (
            //         false,
            //         index.map(|i|
            // definition_ass_list.get(i).unwrap().clone()),     )
            // } else {
            //     (true, None)
            // }
        }
        Expr::Let((name, name_span), value) => {
            let new_decl = Vector::unit((name.clone(), name_span.clone()));
            let next_symbol = match reference_symbol {
                Founding(ident) if *ident >= name_span.start && *ident < name_span.end => {
                    let spanned_name = (name.clone(), name_span.clone());
                    if include_self {
                        reference_list.push(spanned_name.clone());
                    }

                    Cow::Owned(ReferenceSymbol::Founded(spanned_name))
                }
                _ => Cow::Borrowed(reference_symbol),
            };

            get_reference_of_expr(
                value,
                definition_ass_list,
                next_symbol.borrow(),
                reference_list,
                include_self,
            );
        }
        Expr::Then(first, second) => {
            get_reference_of_expr(
                first,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
            get_reference_of_expr(
                second,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
        }
        Expr::Binary(lhs, _op, rhs) => {
            get_reference_of_expr(
                lhs,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
            get_reference_of_expr(
                rhs,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
        }
        Expr::Call(callee, args) => {
            get_reference_of_expr(
                callee,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
            for expr in &args.0 {
                get_reference_of_expr(
                    expr,
                    definition_ass_list,
                    reference_symbol,
                    reference_list,
                    include_self,
                );
            }
        }
        Expr::If(test, consequent, alternative) => {
            get_reference_of_expr(
                test,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
            get_reference_of_expr(
                consequent,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
            get_reference_of_expr(
                alternative,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
        }
        Expr::Print(expr) => get_reference_of_expr(
            expr,
            definition_ass_list,
            reference_symbol,
            reference_list,
            include_self,
        ),
        Expr::List(lst) => {
            for expr in lst {
                get_reference_of_expr(
                    expr,
                    definition_ass_list,
                    reference_symbol,
                    reference_list,
                    include_self,
                );
            }
        }
    }
}
