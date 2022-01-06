use std::rc::Rc;
use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm};
use crate::tiles::TileIndex;
use serde::{Serialize, Deserialize};
use shipyard::{AddEntity, IntoIter, ViewMut, IntoWithId, UniqueViewMut, UniqueView, EntitiesViewMut, Storage, EntityId, Get, Remove};
use simple_ringbuf::RingBuffer;
use crate::colors::named_color;
use crate::entity::{HasPosition, IsDirty};
use crate::game_systems::IsLocked;
use crate::map::Map;

pub struct MoveDirective(pub EntityId, pub Point);

pub fn resolve_move_directives(map: &Map,
                               mut move_dirs: ViewMut<MoveDirective>,
                               mut positions: ViewMut<HasPosition>,
                               mut dirty: UniqueViewMut<IsDirty>,
                               mut entities: EntitiesViewMut) {

    let mut to_be_removed = Vec::new();

    for mov in (&move_dirs).iter() {
        if let Ok(mut pos) = (&mut positions).get(mov.0) {
            pos.0 = mov.1;
            dirty.0 = true;
            to_be_removed.push(mov.0);
        }
    }

    for id in to_be_removed {
        (&mut move_dirs).remove(id);
    }
}

pub struct UnlockDirective(pub EntityId);

pub fn resolve_unlock_directive(mut unlock_dirs: ViewMut<UnlockDirective>,
                                mut is_locked: ViewMut<IsLocked>,
                                mut dirty: UniqueViewMut<IsDirty>,
                                mut entities: EntitiesViewMut) {

    let mut to_be_removed = Vec::new();

    for (id, dir) in (&unlock_dirs).iter().with_id() {
        to_be_removed.push(id);
        (&mut is_locked).remove(dir.0);
        dirty.0 = true;
    }

    for id in to_be_removed {
        unlock_dirs.delete(id);
        unlock_dirs.remove(id);
        entities.delete_unchecked(id);
    }
}