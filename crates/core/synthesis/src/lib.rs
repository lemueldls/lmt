#![feature(try_blocks)]
// #![feature(mapped_lock_guards)]

mod context;
mod graph;
mod module;
mod temp;
mod r#type;
mod util;

use core::fmt;
use std::{
    collections::HashMap,
    fs,
    hash::RandomState,
    ops::{self},
    path::{Path, PathBuf},
    rc::Rc, // sync::{Arc, LazyLock, MappedRwLockReadGuard, RwLock, RwLockReadGuard},
};

pub use context::{Context, scope};
// use ariadne::{Label, Report, ReportKind};
use dashmap::DashMap;
pub use graph::Graph;
use lmt_parser::{Block, Input, Parser, SimpleSpan, Spanned, Stmt, Token, error::Rich};
use lmt_report::{SynthesisReport, miette::SourceSpan};
// use module::Synthesis;
pub use module::{ModuleId, ModuleMap, ModuleSynthesis, SpanWithModuleId};
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
pub use r#type::{Implication, Primitive, Proof, SynType, SynTypeKind, TypeId, TypeMap};
// use parking_lot::{const_rwlock, RwLock, RwLockReadGuard};

#[derive(Debug, Default)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct StaticSynthesis {
    pub sources: DashMap<ModuleId, String>,
    pub module_synthesis_map: ModuleMap<ModuleSynthesis>,
    /// Maps the span of an identifier with the it's corresponding type.
    // pub ident_types: Arc<RwLock<HashMap<SpanWithFileId, Arc<SynType>>>>,
    pub context: context::Context,
}

impl StaticSynthesis {
    // pub const fn new() -> Self {
    //     Self {
    //         sources: RwLock::new(HashMap::with_hasher(RandomState::new())),
    //         module_synthesis_map: todo!(),
    //         context: todo!(),
    //     }
    // }

    pub fn load_module(&self, path: &str, src: &str) -> (ModuleId, Vec<SynthesisReport>) {
        // let src = self
        //     .graph
        //     .write()
        //     .unwrap()
        //     .resolve_path(PathBuf::from(path))
        //     .unwrap();

        let (tokens, tokenize_errors): (
            Option<Vec<(Token, SimpleSpan)>>,
            Vec<Rich<char, SimpleSpan>>,
        ) = lmt_parser::lexer().parse(src).into_output_errors();

        let mut errors: Vec<SynthesisReport> = tokenize_errors
            .into_iter()
            .map(|error| {
                SynthesisReport::TokenizeError {
                    reason: error.reason().to_string(),
                    span: SourceSpan::from(error.span().into_range()),
                }
            })
            .collect();

        let (ast, parse_errors): (Option<Vec<Spanned<Stmt>>>, Vec<Rich<Token, SimpleSpan>>) =
            match &tokens {
                Some(tokens) => {
                    let len = src.chars().count();

                    lmt_parser::module_parser()
                        .parse(tokens.spanned((len..len).into()))
                        .into_output_errors()
                }
                None => (None, Vec::new()),
            };

        errors.extend(parse_errors.into_iter().map(|error| {
            SynthesisReport::TokenizeError {
                reason: error.reason().to_string(),
                span: SourceSpan::from(error.span().into_range()),
            }
        }));

        let module_id = self.module_synthesis_map.push_map(ModuleSynthesis::new);

        // self.module_synthesis_map.write()[module_id.0].eval_statement(stmt, context)
        if let Some(stmts) = ast {
            let block = Block::Multiline {
                return_typed_ident: None,
                stmts,
            };

            // TODO: no write locking
            self.module_synthesis_map.get_mut(module_id).eval_block(
                Spanned::new(block, SimpleSpan::from(0..src.len())),
                true,
                &self.context,
            );
        }

        // self.sources.write().insert(module_id, src);

        (module_id, errors)
    }

    // pub fn register_module_ast(&self, module_id: ModuleId, ast: Option<Vec<Spanned<Stmt>>>) {
    //     // let module_synthesis = ast.map(|stmts| self.synthesize(0, stmts));

    //     // self.module_synthesis_map.write()[module_id.0] = module_synthesis;

    //     self.module_synthesis_map.write()[module_id.0].eval_statement(stmt, context)

    //     // self.sources.write().insert(module_id, src);
    // }

    // pub fn parse_module<'a>(
    //     &'a self,
    //     src: &'a str,
    //     // module_id: ModuleId,
    // ) -> (
    //     Option<Vec<Spanned<Stmt>>>,
    //     // Vec<Rich<char, Span>>,
    //     Vec<Report<(ModuleId, Range<usize>)>>,
    // ) {
    //     let (tokens, tokenize_errors) = lmt_parser::lexer().parse(src).into_output_errors();

    //     let mut errors: Vec<Report<(ModuleId, Range<usize>)>> = tokenize_errors
    //         .into_iter()
    //         .map(|err| {
    //             Report::build(ReportKind::Error, module_id, err.span().start)
    //                 .with_message(err.to_string())
    //                 .with_label(
    //                     Label::new((module_id, err.span().into_range()))
    //                         .with_message(err.reason().to_string()),
    //                 )
    //                 .with_labels(err.contexts().map(|(label, span)| {
    //                     Label::new((module_id, span.into_range()))
    //                         .with_message(format!("while parsing this {label}"))
    //                 }))
    //                 .finish()
    //         })
    //         .collect();

    //     let ast: Option<Option<Vec<Spanned<Stmt>>>> = tokens.map(|tokens| {
    //         let len = src.chars().count();

    //         let (ast, parse_errors): (Option<Vec<Spanned<Stmt>>>, Vec<Rich<Token, Span>>) =
    //             lmt_parser::module_parser()
    //                 .parse(tokens.spanned((len..len).into()))
    //                 .into_output_errors();

    //         errors.extend(parse_errors.into_iter().map(|err| {
    //             Report::build(ReportKind::Error, module_id, err.span().start)
    //                 .with_message(err.to_string())
    //                 .with_label(
    //                     Label::new((module_id, err.span().into_range()))
    //                         .with_message(err.reason().to_string()),
    //                 )
    //                 .with_labels(err.contexts().map(|(label, span)| {
    //                     Label::new((module_id, span.into_range()))
    //                         .with_message(format!("while parsing this {label}"))
    //                 }))
    //                 .finish()
    //         }));

    //         ast
    //     });

    //     (ast.flatten(), errors)
    // }

    // fn add_module_synthesis(&mut self, module_id: u32, module: ModuleSynthesis) {
    //     self.module_synthesis_map.insert(module_id, module);
    // }

    // pub fn synthesize(&self, module_id: u32, stmts: Vec<Spanned<Stmt>>) -> ModuleSynthesis {
    //     let mut module = ModuleSynthesis::new(
    //         module_id,
    //         //  Arc::clone(&self.ident_types)
    //     );

    //     for stmt in stmts {
    //         module.eval_statement(stmt, &self.context);
    //     }

    //     module
    // }
}
