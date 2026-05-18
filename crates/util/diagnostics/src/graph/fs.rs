use std::fs;

use lmt_index::MappedRwLockReadGuard;

use crate::{
    ModuleId, ModuleMap,
    graph::ModuleGraph,
    source::{HasNamedSourceIngredient, NamedSource},
};

pub struct FsGraph {
    pub files: ModuleMap<NamedSource>,
}

impl FsGraph {
    pub fn new() -> Self {
        Self {
            files: ModuleMap::new(),
        }
    }
}

impl ModuleGraph for FsGraph {
    fn register<DB: HasNamedSourceIngredient>(&mut self, db: &DB, name: &str) -> ModuleId {
        let working_path = std::env::current_dir().unwrap();
        let path = working_path.join(name);
        let path = path.canonicalize().unwrap();
        let path_str = path.to_str().unwrap();

        let module_id = self.files.push_map(|module_id| {
            NamedSource::new(
                db,
                format!("file://{path_str}"),
                fs::read_to_string(path_str).unwrap(),
                module_id,
            )
            .unwrap()
        });

        module_id
    }

    fn get<'a>(&'a self, module_id: ModuleId) -> MappedRwLockReadGuard<'a, NamedSource> {
        self.files.get(module_id)
    }
}
