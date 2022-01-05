use bracket_lib::prelude::{Algorithm2D, Point};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, IntoWithId, ViewMut};
use crate::game::Store;
use crate::game_systems::{BumpIntent, MoveDirective};
use crate::map::Map;

pub fn bump__default(map: &Map,
                     mut bump_intents: ViewMut<BumpIntent>,
                     mut moves: ViewMut<MoveDirective>,) {

    for bump in (&bump_intents).iter() {
        let tile = map.point2d_to_index(bump.pos);

        if !map.is_tile_blocked(tile) {
            moves.add_entity(bump.bumper, MoveDirective(bump.pos));
        }
    }

    bump_intents.clear();
}