use core::{fmt, marker::PhantomData};
use std::ops::{Deref, DerefMut};

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
pub use slotmap::new_key_type;
use slotmap::{Key, KeyData, SecondaryMap};

pub struct SecondarySlotLockMap<K: Key, T> {
    map: RwLock<SecondaryMap<K, T>>,
}

impl<K: Key, T: fmt::Debug> fmt::Debug for SecondarySlotLockMap<K, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecondarySlotLockMap")
            .field("map", &self.map)
            .finish()
    }
}

impl<K: Key, T> Default for SecondarySlotLockMap<K, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Key, T> SecondarySlotLockMap<K, T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: RwLock::new(SecondaryMap::new()),
        }
    }

    pub fn get(&self, index: K) -> MappedRwLockReadGuard<T> {
        RwLockReadGuard::map(self.map.read(), |map| &map[index])
    }

    pub fn get_mut(&self, index: K) -> MappedRwLockWriteGuard<T> {
        RwLockWriteGuard::map(self.map.write(), |map| &mut map[index])
    }

    pub fn insert(&self, key: K, value: T) {
        if let Some(previous) = self.map.write().insert(key, value) {
            todo!("huh")
        }
    }
}
