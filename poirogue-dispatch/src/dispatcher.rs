
use std::any::TypeId;
use std::rc::Rc;
use std::vec::Vec;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use event::Event;
use listener::EventListener;

pub struct Dispatcher<E: Event> {
    listeners: HashMap<TypeId, Vec<Rc<EventListener<E>>>>,
}

impl<E: Event> Dispatcher<E> {
    pub fn new() -> Dispatcher<E> {
        Dispatcher { listeners: HashMap::new() }
    }

    pub fn dispatch(&self, event: &mut E) {
        let type_id = TypeId::of::<E>();

        if let Some(listeners) = self.listeners.get(&type_id) {
            for l in listeners.iter() {
                if event.is_propagating() {
                    l.apply(event);
                } else {
                    break;
                }
            }
        }
    }

    pub fn add_listener(&mut self, listener: EventListener<E>) -> Rc<EventListener<E>> {
        let rc: Rc<EventListener<E>> = Rc::new(listener);
        let ret = rc.clone();
        let type_id = TypeId::of::<E>();

        match self.listeners.entry(type_id) {
            Entry::Occupied(mut o) => {
                let el = o.get_mut();
                el.push(rc);
                el.sort_by(|a, b| b.get_priority().cmp(&a.get_priority()));
            }
            Entry::Vacant(v) => { v.insert(vec![rc]); }
        }

        ret
    }

    pub fn remove_listener(&mut self, listener: Rc<EventListener<E>>) {
        let type_id = TypeId::of::<E>();

        if let Entry::Occupied(mut o) = self.listeners.entry(type_id) {
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

    pub fn has_listeners(&self) -> bool {
        let type_id = TypeId::of::<E>();

        self.listeners.contains_key(&type_id) &&
            self.listeners.get(&type_id).map_or(0, |v| v.len()) > 0
    }
}
