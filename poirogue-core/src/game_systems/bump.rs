use bracket_lib::prelude::{Algorithm2D, Point};
use shipyard::{AllStoragesViewMut, EntityId};
use crate::BUMP_INTENT_REQUEST_QUEUE;
use crate::game::Store;
use crate::game_systems::{BumpIntent, MoveDirective};
use crate::map::Map;

pub fn bump__default((map, store): (&Map, &mut Store), mut storage: AllStoragesViewMut) {
    while let Some(item) = store.lpop::<BumpIntent>(BUMP_INTENT_REQUEST_QUEUE, 0) {
        let index = map.point2d_to_index(Point::from(item.pos));

        if let Some(entity) = EntityId::from_inner(item.entity) {
            if !map.is_tile_blocked(index) {
                storage.add_component(entity, (MoveDirective::MoveTo(index),));
            }
        }
    }
}