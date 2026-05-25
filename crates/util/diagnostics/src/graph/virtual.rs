use std::collections::HashMap;

use lmt_index::MappedRwLockReadGuard;

use crate::{
    ModuleId, ModuleMap,
    graph::ModuleGraph,
    source::{HasNamedSourceIngredient, NamedSource},
};

pub struct VirtualGraph {
    pub files: ModuleMap<NamedSource>,
    pub modules_by_name: HashMap<String, ModuleId>,
}

impl Default for VirtualGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualGraph {
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: ModuleMap::new(),
            modules_by_name: HashMap::new(),
        }
    }

    fn source_name(name: &str) -> String {
        if name.contains("://") {
            name.to_string()
        } else {
            format!("virtual://{name}")
        }
    }
}

impl ModuleGraph for VirtualGraph {
    fn upsert<DB: HasNamedSourceIngredient>(
        &mut self,
        db: &DB,
        name: &str,
        content: String,
    ) -> ModuleId {
        if let Some(module_id) = self.modules_by_name.get(name).copied() {
            let source = self.files.get_mut(module_id);
            let source_name = source.name(db).unwrap().to_string();
            db.named_source_data().set(
                db,
                source.0,
                NamedSource::new(db, source_name, content, module_id)
                    .unwrap()
                    .data(db)
                    .unwrap(),
            );
            drop(source);

            return module_id;
        }

        let source_name = Self::source_name(name);
        let module_id = self
            .files
            .push_map(|module_id| NamedSource::new(db, source_name, content, module_id).unwrap());
        self.modules_by_name.insert(name.to_string(), module_id);

        module_id
    }

    fn module_id(&self, name: &str) -> Option<ModuleId> {
        self.modules_by_name.get(name).copied()
    }

    fn get(&self, module_id: ModuleId) -> MappedRwLockReadGuard<'_, NamedSource> {
        self.files.get(module_id)
    }
}
