use std::default::Default as StdDefault;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Dirty<T>(pub T);

#[derive(Debug)]
pub struct Default<T: StdDefault + Clone>(pub T);

impl <T> Reset for Dirty<T> {
    fn reset(&mut self) {
        // Do nothing!
    }
}

impl <T: StdDefault + Clone> Reset for Default<T> {
    fn reset(&mut self) {
        self.0.clone_from(&StdDefault::default());
    }
}

unsafe impl <T: Send> Send for Dirty<T> {}
unsafe impl <T: Sync> Sync for Dirty<T> {}

impl <T> Deref for Dirty<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl <T> DerefMut for Dirty<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

unsafe impl <T: Send + Clone + StdDefault> Send for Default<T> {}
unsafe impl <T: Sync + Clone + StdDefault> Sync for Default<T> {}

impl <T: StdDefault + Clone> Deref for Default<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl <T: StdDefault + Clone> DerefMut for Default<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// Resetting an object reverts that object back to a default state.
pub trait Reset {
    fn reset(&mut self);
}
