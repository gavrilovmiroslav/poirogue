
pub trait Maybe<T> {
    fn then_maybe(self, t: T) -> Option<T>;
}

impl<T> Maybe<T> for bool {
    fn then_maybe(self, t: T) -> Option<T> {
        if self {
            Some(t)
        } else {
            None
        }
    }
}