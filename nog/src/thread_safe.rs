use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct ThreadSafe<T>(Arc<RwLock<T>>);

impl<T> ThreadSafe<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.0.write().unwrap()
    }
}

impl<T> Clone for ThreadSafe<T> {
    fn clone(&self) -> Self {
        ThreadSafe(self.0.clone())
    }
}

impl<T> Default for ThreadSafe<T>
where
    T: Default,
{
    fn default() -> Self {
        ThreadSafe::new(T::default())
    }
}
