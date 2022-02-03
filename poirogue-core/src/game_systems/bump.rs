use std::collections::VecDeque;
use bracket_lib::prelude::{Algorithm2D, Point};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, IntoWithId, Storage, UniqueView, UniqueViewMut, ViewMut};
use crate::game::{Store, Time};
use crate::game_systems::{BumpIntent, MoveDirective, ResolvedIntents};
use crate::map::Map;

pub fn on_bump_move_if_empty(map: UniqueView<Map>,
                             mut bump_intents: UniqueViewMut<VecDeque<BumpIntent>>,
                             mut move_directives: UniqueViewMut<VecDeque<MoveDirective>>,
                             mut handled: UniqueViewMut<ResolvedIntents>,
                             mut time: UniqueViewMut<Time>) {

    for bump in bump_intents.iter().filter(|&&i| handled.not_handled(i)) {
        let tile = map.point2d_to_index(bump.pos);

        if !map.is_tile_blocked(tile) {
            move_directives.push_back(MoveDirective(bump.bumper, bump.pos));
            time.push_current_back(Time::TICK);
        }
    }
}