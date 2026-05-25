//! lmt-index - Indexing utilities for LMT.

mod secondary;

use std::fmt;

pub use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use secondary::SecondarySlotLockMap;
use slotmap::SlotMap;
pub use slotmap::{Key, KeyData};

#[macro_export]
macro_rules! define_index_type {
    ( $(#[$outer:meta])* $vis:vis struct $name:ident; $($rest:tt)* ) => {
        $(#[$outer])*
        #[derive(::facet::Facet, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[facet(opaque, proxy = u64)]
        $vis struct $name($crate::KeyData);

        impl ::std::convert::From<u64> for $name {
            fn from(u: u64) -> Self {
                Self($crate::KeyData::from_ffi(u))
            }
        }

        impl ::std::convert::From<&$name> for u64 {
            fn from(id: &$name) -> Self {
                id.0.as_ffi()
            }
        }


        impl ::std::convert::From<$crate::KeyData> for $name {
            fn from(k: $crate::KeyData) -> Self {
                $name(k)
            }
        }

        unsafe impl $crate::Key for $name {
            fn data(&self) -> $crate::KeyData {
                self.0
            }
        }

        $crate::define_index_type!($($rest)*);
    };

    () => {}
}

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

    pub fn get(&self, index: K) -> MappedRwLockReadGuard<'_, T> {
        RwLockReadGuard::map(self.map.read(), |map| &map[index])
    }

    pub fn get_mut(&self, index: K) -> MappedRwLockWriteGuard<'_, T> {
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
