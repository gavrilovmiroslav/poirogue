
use event::Event;

pub struct EventListener<E: Event> {
    callback: fn(event: &mut E),
    priority: u8,
}

impl<E: Event> EventListener<E> {
    pub fn new(listener: fn(event: &mut E)) -> EventListener<E> {
        EventListener { callback: listener, priority: 0, }
    }

    pub fn new_with_priority(listener: fn(event: &mut E), priority: u8) -> EventListener<E> {
        EventListener { callback: listener, priority, }
    }

    pub fn apply(&self, event: &mut E) {
        (self.callback)(event);
    }

    pub fn get_priority(&self) -> u8 {
        self.priority
    }
}

impl<E: Event> PartialEq for EventListener<E> {
    fn eq(&self, other: &EventListener<E>) -> bool {
        (self.callback as *const ()) == (other.callback as *const ())
    }

    fn ne(&self, other: &EventListener<E>) -> bool {
        !self.eq(other)
    }
}
