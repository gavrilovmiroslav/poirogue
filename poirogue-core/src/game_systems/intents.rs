use std::collections::{HashSet, VecDeque};
use bracket_lib::prelude::Point;
use crate::tiles::TileIndex;
use shipyard::{AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, IntoWithId, Not, SparseSet, Storage, UniqueViewMut, View, ViewMut};


pub fn delete_intents(mut storage: AllStoragesViewMut) {
    storage.borrow::<UniqueViewMut<VecDeque<BumpIntent>>>().unwrap().clear();
    storage.borrow::<UniqueViewMut<VecDeque<UnlockIntent>>>().unwrap().clear();
    storage.borrow::<UniqueViewMut<VecDeque<CollectIntent>>>().unwrap().clear();
    storage.borrow::<UniqueViewMut<VecDeque<InvestigateIntent>>>().unwrap().clear();
    storage.borrow::<UniqueViewMut<ResolvedIntents>>().unwrap().0.clear();
}

#[derive(Default)]
pub struct ResolvedIntents(pub HashSet<u64>);

#[derive(Copy, Clone)]
pub struct BumpIntent {
    pub id: u64,
    pub bumper: EntityId,
    pub pos: Point,
}

#[derive(Copy, Clone)]
pub struct UnlockIntent {
    pub id: u64,
    pub entity: EntityId,
    pub target: EntityId,
}

#[derive(Copy, Clone)]
pub struct CollectIntent {
    pub id: u64,
    pub collector: EntityId,
    pub item: EntityId,
}

#[derive(Copy, Clone)]
pub struct InvestigateIntent {
    pub id: u64,
    pub pos: Point,
}
