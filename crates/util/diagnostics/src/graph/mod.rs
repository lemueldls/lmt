#[cfg(feature = "fs")]
mod fs;
mod lsp;
mod r#virtual;

#[cfg(feature = "fs")]
pub use fs::FsGraph;
use lmt_index::MappedRwLockReadGuard;
pub use lsp::LspGraph;
pub use r#virtual::VirtualGraph;

use crate::{
    ModuleId,
    source::{HasNamedSourceIngredient, NamedSource},
};

/// Graph abstraction for managing modules and their sources, supporting different workflows like
/// filesystem-based, LSP/editor-based, or virtual/in-memory modules.
pub trait ModuleGraph {
    /// Insert or update a module by name and return its stable module id.
    fn upsert<DB: HasNamedSourceIngredient>(
        &mut self,
        db: &DB,
        name: &str,
        content: String,
    ) -> ModuleId;

    /// Resolve a module id by its name if present.
    fn module_id(&self, name: &str) -> Option<ModuleId>;

    /// Get the source of a module by its id.
    fn get<'a>(&'a self, module_id: ModuleId) -> MappedRwLockReadGuard<'a, NamedSource>;
}
