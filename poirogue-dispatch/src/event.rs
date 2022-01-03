
pub struct Envelope<Data: 'static> {
    pub data: Data,
    pub is_propagating: bool,
}

impl<Data: 'static> Envelope<Data> {
    pub fn new(data: Data) -> Envelope<Data> {
        Envelope { data, is_propagating: true }
    }

    pub fn unwrap(self) -> Data {
        self.data
    }
}

pub trait Event<E: ?Sized>: 'static {
    fn is_propagating(&self) -> bool;
    fn stop_propagation(&mut self);
}

impl<Data: 'static> Event<Envelope<Data>> for Envelope<Data> {
    fn is_propagating(&self) -> bool {
        self.is_propagating
    }

    fn stop_propagation(&mut self) {
        self.is_propagating = false;
    }
}
