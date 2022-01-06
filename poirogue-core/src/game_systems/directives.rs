use std::rc::Rc;
use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm};
use crate::tiles::TileIndex;
use serde::{Serialize, Deserialize};
use shipyard::{AddEntity, IntoIter, ViewMut, IntoWithId, UniqueViewMut, UniqueView, EntitiesViewMut, Storage};
use simple_ringbuf::RingBuffer;
use crate::colors::named_color;
use crate::entity::{HasPosition, IsDirty};
use crate::map::Map;

pub struct MoveDirective(pub Point);

pub fn resolve_move_directives(map: &Map,
                               mut positions: ViewMut<HasPosition>,
                               mut move_dirs: ViewMut<MoveDirective>,
                               mut dirty: UniqueViewMut<IsDirty>,) {

    let mut any_resolved = false;
    for (mut pos, mov) in (&mut positions, &move_dirs).iter() {
        pos.0 = mov.0;
        any_resolved = true;
    }

    if any_resolved {
        dirty.0 = true;
    }

    move_dirs.clear();
}
