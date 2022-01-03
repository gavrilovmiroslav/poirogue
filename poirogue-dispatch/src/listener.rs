use crate::event::Event;

pub struct EventListener<E: Event<T>, T: ?Sized>{
    callback: fn(event: &E),
    priority: u8
}

impl<E: Event<T>, T: ?Sized> EventListener<E, T> {
    pub fn new(listener: fn(event: &E)) -> EventListener<E, T> {
        EventListener { callback: listener, priority: 0 }
    }

    pub fn new_with_priority(listener: fn(event: &E), priority: u8) -> EventListener<E, T> {
        EventListener { callback: listener, priority }
    }

    pub fn apply(&self, event: &E) {
        self.callback(event);
    }

    pub fn get_priority(&self) -> u8 {
        self.priority
    }
}

impl<E: Event<T>, T: ?Sized> PartialEq for EventListener<E, T> {
    fn eq(&self, other: &EventListener<E, T>) -> bool {
        (self.callback as *const()) == (other.callback as *const())
    }

    fn ne(&self, other: &EventListener<E, T>) -> bool {
        !self.eq(other)
    }
}
