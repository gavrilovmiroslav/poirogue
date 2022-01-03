use std::any::TypeId;
use std::rc::Rc;
use std::vec::Vec;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::marker::PhantomData;

use crate::event::Event;
use crate::listener::EventListener;

pub trait Dispatch<T: ?Sized, E: Event<T>> {
    fn get_dispatcher(&self) -> &Dispatcher<T, E>;
    fn get_dispatcher_mut(&mut self) -> &mut Dispatcher<T, E>;

    fn dispatch(&self, event: &mut E) {
        let type_id = TypeId::of::<E>();

        if let Some(listeners) = self.get_dispatcher().listeners.get(&type_id) {
            for l in listeners.iter() {
                if event.is_propagating() {
                    l.apply(event);
                } else {
                    break;
                }
            }
        }
    }

    fn add_listener(&mut self, listener: EventListener<T, E>) -> Rc<EventListener<T, E>> {
        let rc: Rc<EventListener<T, E>> = Rc::new(listener);
        let ret = rc.clone();
        let type_id = TypeId::of::<E>();

        match self.get_dispatcher_mut().listeners.entry(type_id) {
            Entry::Occupied(mut o) => {
                let el = o.get_mut();
                el.push(rc);
                el.sort_by(|a, b| b.get_priority().cmp(&a.get_priority()));
            }
            Entry::Vacant(v) => { v.insert(vec![rc]); }
        }

        ret
    }

    fn remove_listener(&mut self, listener: Rc<EventListener<T, E>>) {
        let type_id = TypeId::of::<E>();

        if let Entry::Occupied(mut o) = self.get_dispatcher_mut().listeners.entry(type_id) {
            let listeners = o.get_mut();
            let mut idx: usize = 0;
            for l in listeners.iter() {
                if *l == listener {
                    break;
                }
                idx += 1;
            }

            listeners.remove(idx);
        }
    }
}

pub struct Dispatcher<T: ?Sized, E: Event<T>> {
    listeners: HashMap<TypeId, Vec<Rc<EventListener<T, E>>>>,
    phantom: PhantomData<T>,
}

impl<T: ?Sized, E: Event<T>> Dispatcher<T, E> {
    pub fn new () -> Dispatcher<T, E> {
        Dispatcher{ listeners: HashMap::new(), phantom: PhantomData }
    }

    pub fn has_listeners(&self) -> bool {
        let type_id = TypeId::of::<E>();

        self.listeners.contains_key(&type_id) &&
            self.listeners.get(&type_id).map_or(0, |v| v.len()) > 0
    }
}

impl<T: ?Sized, E: Event<T>> Dispatch<T, E> for Dispatcher<T, E> {
    fn get_dispatcher(&self) -> &Dispatcher<T, E> { self }
    fn get_dispatcher_mut(&mut self) -> &mut Dispatcher<T, E> { self }
}