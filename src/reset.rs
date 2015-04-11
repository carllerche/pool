use std::collections::*;

/// Resetting an object that acts like a collection clears the contents of
/// that collection and makes it so that collection behaves like it was not
/// used before.
pub trait Reset {
    fn reset(&mut self);
}

impl <T> Reset for Vec<T> {
    fn reset(&mut self) {
        self.clear();
    }
}

impl <T: Ord> Reset for BinaryHeap<T> {
    fn reset(&mut self) {
        self.clear();
    }
}

impl <T> Reset for LinkedList<T> {
    fn reset(&mut self) {
        self.clear();
    }
}

impl <T> Reset for VecDeque<T> {
    fn reset(&mut self) {
        self.clear();
    }
}

impl Reset for String {
    fn reset(&mut self) {
        self.clear();
    }
}

// TODO: uncomment when VecMap becomes stable
/*
impl <T> Reset for VecMap<T> {
    fn reset(&mut self) {
        self.clear();
    }
}
*/
