use bracket_lib::prelude::Point;
use crate::tiles::TileIndex;
use shipyard::{AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, IntoWithId, Not, SparseSet, Storage, View, ViewMut};

pub struct Handle<T> {
    pub intent: T,
    pub handled: bool,
    pub origin: Vec<EntityId>,
}

impl<T> Handle<T> {
    pub fn new(intent: T) -> Self {
        Handle { intent, handled: false, origin: Vec::new() }
    }

    pub fn spawn<N>(&self, self_id: EntityId, intent: N) -> Handle<N> {
        let mut origin = self.origin.clone();
        origin.push(self_id);
        Handle { intent, handled: false, origin }
    }
}

pub fn propagate_handled_intents<T: 'static + Send + Sync>(mut storage: &mut AllStoragesViewMut,) {
    let mut deleted = Vec::new();
    for (id, h) in (&storage.borrow::<View<Handle<T>>>().unwrap())
        .iter().with_id().filter(|(_, h)| h.handled) {

        deleted.push(id);
        for prev in &h.origin {
            deleted.push(*prev);
        }
    }

    for id in deleted {
        storage.delete_entity(id);
    }
}

pub fn propagate_handled_and_delete_rest<T: 'static + Send + Sync>(mut storage: &mut AllStoragesViewMut) {
    propagate_handled_intents::<T>(&mut storage);

    let mut deleted = Vec::new();
    for (id, _) in (&storage.borrow::<View<Handle<T>>>().unwrap()).iter().with_id() {
        deleted.push(id);
    }

    for id in deleted {
        let _ = &storage.borrow::<ViewMut<Handle<T>>>().unwrap().delete(id);
        let _ = &storage.borrow::<ViewMut<Handle<T>>>().unwrap().remove(id);
        let _ = &storage.borrow::<EntitiesViewMut>().unwrap().delete_unchecked(id);
    }

    storage.delete_any::<SparseSet<Handle<T>>>();
}

pub fn delete_handled_intents(mut storage: AllStoragesViewMut) {
    propagate_handled_intents::<BumpIntent>(&mut storage);
    propagate_handled_intents::<UnlockIntent>(&mut storage);
    propagate_handled_intents::<CollectIntent>(&mut storage);
    propagate_handled_intents::<InvestigateIntent>(&mut storage);
}


pub struct BumpIntent {
    pub bumper: EntityId,
    pub pos: Point,
}

pub struct UnlockIntent {
    pub entity: EntityId,
    pub target: EntityId,
}

pub struct CollectIntent {
    pub collector: EntityId,
    pub item: EntityId,
}

pub struct InvestigateIntent {
    pub pos: Point,
}
