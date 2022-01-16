use bracket_lib::prelude::{Algorithm2D, Point};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, IntoWithId, Storage, UniqueView, UniqueViewMut, ViewMut};
use crate::game::Store;
use crate::game_systems::{BumpIntent, Handle, MoveDirective};
use crate::map::Map;

pub fn on_bump_default(map: UniqueView<Map>,
                       mut bump_intents: ViewMut<Handle<BumpIntent>>,
                       mut moves: ViewMut<MoveDirective>,) {

    for mut bump in (&mut bump_intents).iter()
        .filter(|b| !b.handled) {

        let tile = map.point2d_to_index(bump.intent.pos);

        if !map.is_tile_blocked(tile) {
            moves.add_entity(bump.intent.bumper, MoveDirective(bump.intent.bumper, bump.intent.pos));
        }

        bump.handled = true;
    }
}