use core::{fmt, marker::PhantomData};

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

pub trait LockIndex: Copy {
    fn as_usize(&self) -> usize;
    fn from_usize(n: usize) -> Self;
}

#[macro_export]
macro_rules! define_index_type {
    ($vis:vis $name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        // #[cfg_attr(feature = "serde", derive(bitcode::Encode, bitcode::Decode))]
        $vis struct $name(usize);

        impl $crate::util::LockIndex for $name {
          fn as_usize(&self) -> usize {
            self.0
          }

          fn from_usize(n: usize) -> Self {
            Self(n)
          }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

pub struct SlotLockMap<I: LockIndex, T> {
    vec: RwLock<Vec<T>>,
    marker: PhantomData<I>,
}

impl<I: LockIndex, T: fmt::Debug> fmt::Debug for SlotLockMap<I, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SlotLockMap")
            .field("vec", &self.vec)
            .field("marker", &self.marker)
            .finish()
    }
}

impl<I: LockIndex, T> Default for SlotLockMap<I, T> {
    fn default() -> Self {
        Self {
            vec: RwLock::new(Vec::new()),
            marker: PhantomData,
        }
    }
}

impl<I: LockIndex, T> SlotLockMap<I, T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self {
            vec: RwLock::new(vec),
            marker: PhantomData,
        }
    }

    pub fn get(&self, index: I) -> MappedRwLockReadGuard<T> {
        RwLockReadGuard::map(self.vec.read(), |vec| &vec[index.as_usize()])
    }

    pub fn get_mut(&self, index: I) -> MappedRwLockWriteGuard<T> {
        RwLockWriteGuard::map(self.vec.write(), |vec| &mut vec[index.as_usize()])
    }

    pub fn push(&self, value: T) -> I {
        let mut guard = self.vec.write();
        let id = I::from_usize(guard.len());

        guard.push(value);

        id
    }

    pub fn push_map(&self, f: impl FnOnce(I) -> T) -> I {
        // let guard = self.vec.write();
        // let index = guard.len();

        // let id = I::from_usize(index);

        // let value = f(id);

        // self.vec.write()[index] = value;

        // id

        let mut guard = self.vec.write();
        let id = I::from_usize(guard.len());

        guard.push(f(id));

        id
    }

    pub fn with_capacity_from(other: &Self) -> Self {
        Self::new(Vec::with_capacity(other.vec.read().len()))
    }
}

// impl<T + Default, I: Into<usize>> ops::Deref for SlotLockMap<I, T> {
//     type Target = RwLock<Vec<T>>;

//     fn deref(&self) -> &Self::Target {
//         &self.vec
//     }
// }

// impl<T + Default, I: Into<usize>> ops::DerefMut for SlotLockMap<I, T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.vec
//     }
// }
