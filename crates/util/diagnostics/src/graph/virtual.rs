use lmt_index::MappedRwLockReadGuard;
use picante::PicanteResult;

use crate::{
    ModuleId, ModuleMap,
    graph::ModuleGraph,
    source::{HasNamedSourceIngredient, NamedSource},
};

pub struct VirtualGraph {
    pub files: ModuleMap<NamedSource>,
}

impl VirtualGraph {
    pub fn new() -> Self {
        Self {
            files: ModuleMap::new(),
        }
    }
}

impl ModuleGraph for VirtualGraph {
    fn register<DB: HasNamedSourceIngredient>(&mut self, db: &DB, name: &str) -> ModuleId {
        let module_id = self.files.push_map(|module_id| {
            NamedSource::new(db, format!("virtual://{name}"), String::new(), module_id).unwrap()
        });

        module_id
    }

    fn get<'a>(&'a self, module_id: ModuleId) -> MappedRwLockReadGuard<'a, NamedSource> {
        self.files.get(module_id)
    }
}

impl VirtualGraph {
    pub fn set_content<DB: HasNamedSourceIngredient>(
        &mut self,
        db: &DB,
        module_id: ModuleId,
        content: String,
    ) -> PicanteResult<()> {
        let source = self.files.get_mut(module_id);
        let name = source.name(db).unwrap().to_string();
        db.named_source_data().set(
            db,
            source.0,
            NamedSource::new(db, name, content, module_id)?.data(db)?,
        );

        Ok(())
    }
}
