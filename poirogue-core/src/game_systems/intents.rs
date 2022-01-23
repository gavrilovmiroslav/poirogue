use std::collections::{HashSet, VecDeque};
use bracket_lib::prelude::Point;
use crate::tiles::TileIndex;
use shipyard::{AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, IntoWithId, Not, SparseSet, Storage, UniqueViewMut, View, ViewMut};

#[derive(Hash, Eq, PartialEq)]
pub enum PlannedIntent {
    Bump(BumpIntent),
    Unlock(UnlockIntent),
    Collect(CollectIntent),
}

#[derive(Hash, Eq, PartialEq)]
pub struct Intent {
    pub speed: u8,
    pub plan: PlannedIntent,
}

pub fn delete_intents(mut storage: AllStoragesViewMut) {
    storage.borrow::<UniqueViewMut<VecDeque<BumpIntent>>>().unwrap().clear();
    storage.borrow::<UniqueViewMut<VecDeque<UnlockIntent>>>().unwrap().clear();
    storage.borrow::<UniqueViewMut<VecDeque<CollectIntent>>>().unwrap().clear();
    storage.borrow::<UniqueViewMut<ResolvedIntents>>().unwrap().0.clear();
}

pub trait Identifiable {
    fn id(&self) -> u64;
}

#[derive(Default)]
pub struct ResolvedIntents(pub HashSet<u64>);

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct BumpIntent {
    pub id: u64,
    pub bumper: EntityId,
    pub pos: Point,
}

impl Identifiable for BumpIntent { fn id(&self) -> u64 { self.id } }

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct UnlockIntent {
    pub id: u64,
    pub entity: EntityId,
    pub target: EntityId,
}

impl Identifiable for UnlockIntent { fn id(&self) -> u64 { self.id } }

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct CollectIntent {
    pub id: u64,
    pub collector: EntityId,
    pub item: EntityId,
}

impl Identifiable for CollectIntent { fn id(&self) -> u64 { self.id } }