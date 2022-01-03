use std::any::TypeId;
use std::rc::Rc;
use std::vec::Vec;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::event::Event;
use crate::listener::EventListener;

pub trait Dispatch<E: Event<T>, T: ?Sized> {

    fn get_events(&self) -> &Dispatcher<E, T>;
    fn get_events_mut(&mut self) -> &mut Dispatcher<E, T>;

    fn dispatch(&self, event: &E) {
        let type_id = TypeId::of::<E>();

        if let Some(listeners) = self.get_events().listeners.get(&type_id) {
            for l in listeners.iter() {
                if event.is_bubbling() {
                    l.apply(event);
                } else {
                    break;
                }
            }
        }
    }

    fn add_listener(&mut self, _event: &E, listener: EventListener<E, T>) -> Rc<EventListener<E, T>> {
        let rc: Rc<EventListener<E, T>> = Rc::new(listener);
        let ret = rc.clone();
        let type_id = TypeId::of::<E>();
        match self.get_events_mut().listeners.entry(type_id) {
            Entry::Occupied(mut o) => {
                let mut el = o.get_mut();
                el.push(rc);
                el.sort_by(|a, b| b.get_priority().cmp(&a.get_priority()));
            }
            Entry::Vacant(v) => { v.insert(vec![rc]); }
        }

        ret
    }

    fn remove_listener(&mut self, _event: &E, listener: Rc<EventListener<E, T>>) {
        let type_id = TypeId::of::<E>();

        if let Entry::Occupied(mut o) = self.get_events_mut().listeners.entry(type_id) {
            let mut listeners = o.get_mut();
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

    fn has_listeners(&self, _event: &E) -> bool {
        let type_id = TypeId::of::<E>();
        self.get_events().has_listeners(&type_id)
    }
}

pub struct Dispatcher<E: Event<T>, T: ?Sized> {
    listeners: HashMap<TypeId, Vec<Rc<EventListener<E, T>>>>
}

impl<E: Event<T>, T: ?Sized> Dispatcher<E, T> {
    pub fn new () -> Dispatcher<E, T> {
        Dispatcher{ listeners: HashMap::new() }
    }

    pub fn has_listeners(&self, type_id: &TypeId) -> bool {
        self.listeners.contains_key(&type_id) &&
            self.listeners.get(&type_id).map_or(0, |v| v.len()) > 0
    }
}

impl<E: Event<T>, T: ?Sized> Dispatch<E, T> for Dispatcher<E, T> {
    fn get_events(&self) -> &Dispatcher<E, T> { self }
    fn get_events_mut(&mut self) -> &mut Dispatcher<E, T> { self }
}