use bracket_lib::prelude::{Algorithm2D, Point};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, IntoWithId, Storage, ViewMut};
use crate::game::Store;
use crate::game_systems::{BumpIntent, MoveDirective};
use crate::map::Map;

pub fn on_bump_default(map: &Map,
                       mut bump_intents: ViewMut<BumpIntent>,
                       mut moves: ViewMut<MoveDirective>,
                       mut entities: EntitiesViewMut) {

    let mut ids = Vec::new();
    for (id, bump) in (&bump_intents).iter().with_id() {
        let tile = map.point2d_to_index(bump.pos);

        if !map.is_tile_blocked(tile) {
            moves.add_entity(bump.bumper, MoveDirective(bump.pos));
        }

        ids.push(id);
    }

    for id in ids {
        entities.delete(id);
    }

    bump_intents.clear();
}