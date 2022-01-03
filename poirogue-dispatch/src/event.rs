
pub trait Event<E: ?Sized>: 'static {
    fn is_propagating(&self) -> bool;
    fn stop_propagation(&mut self);
}
