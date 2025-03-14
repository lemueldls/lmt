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
use lmt_parser::{
    Block, Input, Parser, SimpleSpan, Spanned, Stmt, Token, error::Rich, parse_tokens,
};
use lmt_report::{SynthesisReport, miette::SourceSpan};
// use module::Synthesis;
pub use module::{ModuleId, ModuleMap, ModuleSynthesis, SpanWithModuleId};
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
pub use r#type::{Implication, Primitive, Proof, SynType, SynTypeKind, TypeId, TypeMap};
// use parking_lot::{const_rwlock, RwLock, RwLockReadGuard};

#[derive(Debug, Default)]
// #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
pub struct StaticSynthesis {
    // pub sources: DashMap<ModuleId, String>,
    pub module_synthesis_map: ModuleMap<ModuleSynthesis>,
    /// Maps the span of an identifier with the it's corresponding type.
    // pub ident_types: Arc<RwLock<HashMap<SpanWithFileId, Arc<SynType>>>>,
    pub context: context::Context,

    pub graph: DashMap<String, ModuleId>,
}

impl StaticSynthesis {
    // pub const fn new() -> Self {
    //     Self {
    //         sources: RwLock::new(HashMap::with_hasher(RandomState::new())),
    //         module_synthesis_map: todo!(),
    //         context: todo!(),
    //     }
    // }

    pub fn load_module(&self, path: &str, content: &str) -> (ModuleId, Vec<SynthesisReport>) {
        let (ast, errors) = parse_tokens(content);

        self.load_block(path, ast, errors)
    }

    pub fn load_block(
        &self,
        path: &str,
        ast: Option<Spanned<Block>>,
        errors: Vec<SynthesisReport>,
    ) -> (ModuleId, Vec<SynthesisReport>) {
        let module_id = self.module_synthesis_map.push_map(ModuleSynthesis::new);

        self.graph.insert(path.to_string(), module_id);

        if let Some(block) = ast {
            // TODO: no write locking
            self.module_synthesis_map
                .get_mut(module_id)
                .eval_block(block, true, &self.context);
        }

        // self.sources.write().insert(module_id, src);

        (module_id, errors)
    }

    pub fn get_module_synthesis_by_id(
        &self,
        module_id: ModuleId,
    ) -> MappedRwLockReadGuard<ModuleSynthesis> {
        self.module_synthesis_map.get(module_id)
    }

    pub fn get_module_synthesis_by_path(
        &self,
        path: &str,
    ) -> Option<MappedRwLockReadGuard<ModuleSynthesis>> {
        let module_id = self.graph.get(path);

        module_id.map(|module_id| self.get_module_synthesis_by_id(*module_id.value()))
    }
}
