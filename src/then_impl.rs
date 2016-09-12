//! Helper method that I wanted on Result and Option.

pub trait Then<T> {
    fn then<F>(self, op: F) where F: FnOnce(T);
}

impl<T,E> Then<T> for Result<T, E> {
    fn then<F>(self, op: F) where F: FnOnce(T) {
        if self.is_err() {
            return;
        }
        (op)(self.ok().unwrap());
    }
}

impl<T> Then<T> for Option<T> {
    fn then<F>(self, op: F) where F: FnOnce(T) {
        if self.is_none() {
            return;
        }
        (op)(self.unwrap());
    }
}
