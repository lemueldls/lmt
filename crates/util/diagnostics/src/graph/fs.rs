use std::{collections::HashMap, fs};

use lmt_index::MappedRwLockReadGuard;

use crate::{
    ModuleId, ModuleMap,
    graph::ModuleGraph,
    source::{HasNamedSourceIngredient, NamedSource},
};

pub struct FsGraph {
    pub files: ModuleMap<NamedSource>,
    pub modules_by_name: HashMap<String, ModuleId>,
}

impl Default for FsGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FsGraph {
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: ModuleMap::new(),
            modules_by_name: HashMap::new(),
        }
    }

    fn normalize_path(name: &str) -> String {
        let working_path = std::env::current_dir().unwrap();
        let path = working_path.join(name);
        let path = path.canonicalize().unwrap();
        path.to_str().unwrap().to_string()
    }
}

impl ModuleGraph for FsGraph {
    fn upsert<DB: HasNamedSourceIngredient>(
        &mut self,
        db: &DB,
        name: &str,
        content: String,
    ) -> ModuleId {
        let normalized = Self::normalize_path(name);

        if let Some(module_id) = self.modules_by_name.get(&normalized).copied() {
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

        let uri = format!("file://{normalized}");
        let module_id = self
            .files
            .push_map(|module_id| NamedSource::new(db, uri, content, module_id).unwrap());

        self.modules_by_name.insert(normalized, module_id);

        module_id
    }

    fn module_id(&self, name: &str) -> Option<ModuleId> {
        let normalized = Self::normalize_path(name);
        self.modules_by_name.get(&normalized).copied()
    }

    fn get(&self, module_id: ModuleId) -> MappedRwLockReadGuard<'_, NamedSource> {
        self.files.get(module_id)
    }
}

impl FsGraph {
    /// Insert or update a module from a filesystem path.
    ///
    /// # Panics
    ///
    /// Panics if the current working directory cannot be
    /// obtained, if the given path cannot be canonicalized, or if the file at
    /// the resolved path cannot be read as a string.
    pub fn upsert_path<DB: HasNamedSourceIngredient>(&mut self, db: &DB, name: &str) -> ModuleId {
        let normalized = Self::normalize_path(name);
        let content = fs::read_to_string(&normalized).unwrap();

        self.upsert(db, &normalized, content)
    }
}
