use std::collections::VecDeque;
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
pub struct UnlockDirective(pub EntityId);

pub fn resolve_move_directives(mut move_dirs: UniqueViewMut<VecDeque<MoveDirective>>,
                               mut positions: ViewMut<HasPosition>,
                               mut dirty: UniqueViewMut<IsDirty>,) {

    while let Some(mov) = move_dirs.pop_back() {
        if let Ok(mut pos) = (&mut positions).get(mov.0) {
            pos.0 = mov.1;
            dirty.0 = true;
        }
    }
}

pub fn resolve_unlock_directive(mut unlock_dirs: UniqueViewMut<VecDeque<UnlockDirective>>,
                                mut is_locked: ViewMut<IsLocked>,
                                mut dirty: UniqueViewMut<IsDirty>,) {

    while let Some(dir) = unlock_dirs.pop_back() {
        (&mut is_locked).remove(dir.0);
        dirty.0 = true;
    }
}