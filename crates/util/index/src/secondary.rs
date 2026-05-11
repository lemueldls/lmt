use core::fmt;

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use slotmap::{Key, SecondaryMap};

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

    pub fn get<'a>(&'a self, index: K) -> MappedRwLockReadGuard<'a, T> {
        RwLockReadGuard::map(self.map.read(), |map| &map[index])
    }

    pub fn get_mut<'a>(&'a self, index: K) -> MappedRwLockWriteGuard<'a, T> {
        RwLockWriteGuard::map(self.map.write(), |map| &mut map[index])
    }

    pub fn insert(&self, key: K, value: T) {
        if let Some(_previous) = self.map.write().insert(key, value) {
            todo!("huh")
        }
    }
}
