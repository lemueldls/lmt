use crate::{define_index_type, util::SlotLockMap};

define_index_type! { pub ModuleId }

pub type ModuleMap<T> = SlotLockMap<ModuleId, T>;
