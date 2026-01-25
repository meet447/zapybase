//! Synchronization primitives abstraction for cross-platform support
//!
//! This module provides a unified interface for synchronization primitives
//! that work across native (with threading) and WASM (single-threaded) targets.

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
pub use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(any(not(feature = "parallel"), target_arch = "wasm32"))]
mod single_threaded {
    use std::cell::{Ref, RefCell, RefMut};
    use std::ops::{Deref, DerefMut};

    /// A single-threaded RwLock implementation using RefCell
    /// Used when the `parallel` feature is disabled or on WASM targets
    pub struct RwLock<T> {
        inner: RefCell<T>,
    }

    impl<T> RwLock<T> {
        pub fn new(value: T) -> Self {
            Self {
                inner: RefCell::new(value),
            }
        }

        pub fn read(&self) -> RwLockReadGuard<'_, T> {
            RwLockReadGuard {
                inner: self.inner.borrow(),
            }
        }

        pub fn write(&self) -> RwLockWriteGuard<'_, T> {
            RwLockWriteGuard {
                inner: self.inner.borrow_mut(),
            }
        }
    }

    pub struct RwLockReadGuard<'a, T> {
        inner: Ref<'a, T>,
    }

    impl<'a, T> Deref for RwLockReadGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    pub struct RwLockWriteGuard<'a, T> {
        inner: RefMut<'a, T>,
    }

    impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
}

#[cfg(any(not(feature = "parallel"), target_arch = "wasm32"))]
pub use single_threaded::{RwLock, RwLockReadGuard, RwLockWriteGuard};
