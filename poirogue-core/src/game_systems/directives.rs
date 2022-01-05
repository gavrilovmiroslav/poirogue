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
                               mut dirty: ViewMut<IsDirty>,
                               mut move_dirs: ViewMut<MoveDirective>,) {

    for (id, (mut pos, mov)) in (&mut positions, &move_dirs).iter().with_id() {
        pos.0 = mov.0;
        dirty.add_entity(id, IsDirty);
    }

    move_dirs.clear();
}
