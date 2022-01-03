
pub trait Event<E: ?Sized>: 'static {
    fn is_bubbling(&self) -> bool;
    fn stop_bubbling(&mut self);
}

pub struct BaseEvent {
    is_bubbling: bool
}

impl BaseEvent{
    fn new() -> BaseEvent{
        BaseEvent{ is_bubbling: true }
    }
}

impl Event<BaseEvent> for BaseEvent {
    fn is_bubbling(&self) -> bool{
        self.is_bubbling
    }

    fn stop_bubbling(&mut self) {
        self.is_bubbling = false;
    }
}