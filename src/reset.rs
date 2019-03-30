use std::default::Default;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Dirty<T>(pub T);

impl <T> Reset for Dirty<T> {
    fn reset_on_checkout(&mut self) {
        // Do nothing!
    }

    fn reset_on_checkin(&mut self) {
        ();
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

/// Resetting an object reverts that object back to a default state.
pub trait Reset {
    fn reset_on_checkout(&mut self);
    fn reset_on_checkin(&mut self);
}

/// Default `Reset` behaviour for types which don't implement it: Reset only during checkout.
impl <T: Default + Clone> Reset for T {
    fn reset_on_checkout(&mut self) {
        // For most of the stdlib collections, this will "clear" the collection
        // without deallocating.
        self.clone_from(&Default::default());
    }

    fn reset_on_checkin(&mut self) {
        ();
    }
}
