use std::marker::PhantomData;
use crate::event::Event;

pub struct EventListener<T: ?Sized, E: Event<T>> {
    callback: fn(event: &mut E),
    priority: u8,
    phantom: PhantomData<T>,
}

impl<T: ?Sized, E: Event<T>> EventListener<T, E> {
    pub fn new(listener: fn(event: &mut E)) -> EventListener<T, E> {
        EventListener { callback: listener, priority: 0, phantom: PhantomData }
    }

    pub fn new_with_priority(listener: fn(event: &mut E), priority: u8) -> EventListener<T, E> {
        EventListener { callback: listener, priority, phantom: PhantomData }
    }

    pub fn apply(&self, event: &mut E) {
        (self.callback)(event);
    }

    pub fn get_priority(&self) -> u8 {
        self.priority
    }
}

impl<T: ?Sized, E: Event<T>> PartialEq for EventListener<T, E> {
    fn eq(&self, other: &EventListener<T, E>) -> bool {
        (self.callback as *const()) == (other.callback as *const())
    }

    fn ne(&self, other: &EventListener<T, E>) -> bool {
        !self.eq(other)
    }
}
