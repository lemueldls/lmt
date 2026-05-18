use lmt_index::{SecondarySlotLockMap, SlotLockMap, define_index_type};

define_index_type! {
    pub struct ModuleId;
}

pub type ModuleMap<T> = SlotLockMap<ModuleId, T>;
pub type SecondaryModuleMap<T> = SecondarySlotLockMap<ModuleId, T>;
