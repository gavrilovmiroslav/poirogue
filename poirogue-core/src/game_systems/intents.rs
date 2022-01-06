use bracket_lib::prelude::Point;
use crate::tiles::TileIndex;
use shipyard::{EntitiesViewMut, EntityId, IntoIter, IntoWithId, Storage, ViewMut};

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
    pub entity: EntityId,
}

pub struct Handle<T> {
    pub intent: T,
    pub handled: bool,
}

impl<T> Handle<T> {
    pub fn new(intent: T) -> Self {
        Handle { intent, handled: false }
    }
}

fn delete_handled_intent<T: 'static>(mut intents: ViewMut<Handle<T>>,
                                mut entities: &mut EntitiesViewMut,) {

    let mut deleted = Vec::new();
    for (id, _) in (&intents).iter().with_id().filter(|(_, h)| h.handled) {
        deleted.push(id);
    }

    for id in deleted {
        intents.delete(id);
        intents.remove(id);
        entities.delete_unchecked(id);
    }
}

pub fn delete_handled_intents(
    mut bump: ViewMut<Handle<BumpIntent>>,
    mut unlock: ViewMut<Handle<UnlockIntent>>,
    mut collect: ViewMut<Handle<CollectIntent>>,
    mut investigate: ViewMut<Handle<InvestigateIntent>>,
    mut entities: EntitiesViewMut) {

    delete_handled_intent(bump, &mut entities);
    delete_handled_intent(unlock, &mut entities);
    delete_handled_intent(collect, &mut entities);
    delete_handled_intent(investigate, &mut entities);
}