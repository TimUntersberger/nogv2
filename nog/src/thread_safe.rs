use std::sync::{Arc, RwLock};

pub struct ThreadSafe<T>(Arc<RwLock<T>>);

impl<T> Clone for ThreadSafe<T> {
    fn clone(&self) -> Self {
        ThreadSafe(self.0.clone())
    }
}

impl<T> Default for ThreadSafe<T> where T: Default {
    fn default() -> Self {
        ThreadSafe(Arc::new(RwLock::new(T::default())))
    }
}

impl<T> std::ops::Deref for ThreadSafe<T> {
    type Target = Arc<RwLock<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
