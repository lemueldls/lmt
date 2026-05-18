#[cfg(feature = "fs")]
mod fs;
mod r#virtual;

#[cfg(feature = "fs")]
pub use fs::FsGraph;
use lmt_index::MappedRwLockReadGuard;
pub use r#virtual::VirtualGraph;

use crate::{
    ModuleId,
    source::{HasNamedSourceIngredient, NamedSource},
};

pub trait ModuleGraph {
    fn register<DB: HasNamedSourceIngredient>(&mut self, db: &DB, name: &str) -> ModuleId;

    fn get<'a>(&'a self, module_id: ModuleId) -> MappedRwLockReadGuard<'a, NamedSource>;
}
