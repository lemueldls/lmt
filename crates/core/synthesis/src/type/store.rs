use crate::{define_index_type, util::SlotLockMap};

define_index_type! { pub TypeId }

pub type TypeMap<T> = SlotLockMap<TypeId, T>;
