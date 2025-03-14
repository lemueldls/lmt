mod secondary;

use core::{fmt, marker::PhantomData};
use std::ops::{Deref, DerefMut};

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
pub use secondary::SecondarySlotLockMap;
pub use slotmap::new_key_type;
use slotmap::{Key, KeyData, SecondaryMap, SlotMap};

pub struct SlotLockMap<K: Key, T> {
    map: RwLock<SlotMap<K, T>>,
}

impl<K: Key, T: fmt::Debug> fmt::Debug for SlotLockMap<K, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SlotLockMap")
            .field("map", &self.map)
            .finish()
    }
}

impl<K: Key, T> Default for SlotLockMap<K, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Key, T> SlotLockMap<K, T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: RwLock::new(SlotMap::with_key()),
        }
    }

    pub fn get(&self, index: K) -> MappedRwLockReadGuard<T> {
        RwLockReadGuard::map(self.map.read(), |map| &map[index])
    }

    pub fn get_mut(&self, index: K) -> MappedRwLockWriteGuard<T> {
        RwLockWriteGuard::map(self.map.write(), |map| &mut map[index])
    }

    pub fn push(&self, value: T) -> K {
        self.map.write().insert(value)
    }

    pub fn push_map(&self, f: impl FnOnce(K) -> T) -> K {
        self.map.write().insert_with_key(f)
    }

    pub fn create_secondary(&self) -> SecondarySlotLockMap<K, T> {
        SecondarySlotLockMap::new()
    }
}
