use bracket_lib::prelude::{Point, Algorithm2D};
use crate::tiles::TileIndex;
use serde::{Serialize, Deserialize};
use shipyard::{IntoIter, ViewMut};
use crate::entity::{HasPosition, IsDirty};
use crate::map::Map;

#[derive(Serialize, Deserialize)]
pub enum MoveDirective {
    MoveBy(i32, i32),
    MoveTo(TileIndex),
}

pub fn resolve_move_directive(map: &Map, mut positions: ViewMut<HasPosition>, mut dirty: ViewMut<IsDirty>, mut move_dirs: ViewMut<MoveDirective>) {
    for (mut pos, mut dirt, mov) in (&mut positions, &mut dirty, &move_dirs).iter() {
        use MoveDirective::*;

        let new_pos = match mov {
            MoveTo(index) => map.index_to_point2d(*index),
            MoveBy(dx, dy) => pos.0 + Point::from((*dx, *dy)),
        };

        pos.0 = new_pos;
        dirt.mark();
    }

    move_dirs.clear();
}
