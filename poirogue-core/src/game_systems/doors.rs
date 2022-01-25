use std::collections::VecDeque;
use bracket_color::prelude::{ColorPair, HSV};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm, RED, DARK_RED, DARK_GRAY, GOLD, DARK_GOLDENROD, CRIMSON, BLACK, WHITE, ORANGE, DARK_ORANGE, YELLOW, GREEN, DARK_GREEN};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Remove, SparseSet, Storage, UniqueView, UniqueViewMut, View, ViewMut};
use crate::colors::{ColorShifter, named_color, Color};
use crate::entity::{HasSight, HasGlyph, HasPosition, IsDirty, IsPlayer, IsDoor, IsKnown};
use crate::game::{Batch, Store};
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::{CarriesItem, IsItem, NotificationLog, notify, notify_if_alive, NotifyDirective, ResolvedIntents, UnlockDirective};
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::map::Map;
use crate::MAP_CONSOLE_LAYER;
use crate::tiles::{MapTile, TileIndex};

pub fn render_doors(mut batch: UniqueViewMut<Batch>,
                    doors: View<IsDoor>,
                    has_position: View<HasPosition>,
                    has_glyph: View<HasGlyph>,
                    has_sight: View<HasSight>,
                    is_player: View<IsPlayer>,) {

    if let Some((sight, _)) = (&has_sight, &is_player).iter().take(1).next() {
        batch.0.target(MAP_CONSOLE_LAYER);

        for (door, pos, glyph) in (&doors, &has_position, &has_glyph).iter() {
            if sight.field_of_view.contains(&pos.0) {
                let mut fg = glyph.0.fg;

                if door.is_locked.is_some() { fg = fg.lerp(named_color(RED), 0.5); }
                batch.0.set(pos.0, ColorPair::new(fg, named_color(BLACK)), glyph.0.ch as u16);
            }
        }
    }
}

pub fn on_bump_interpret_as_door_unlock_intent(doors: View<IsDoor>,
                                               has_position: View<HasPosition>,
                                               mut bump_intents: UniqueViewMut<VecDeque<BumpIntent>>,
                                               mut unlock_intents: UniqueViewMut<VecDeque<UnlockIntent>>,
                                               mut handled: UniqueViewMut<ResolvedIntents>) {

    for bump in bump_intents.iter().filter(|i| !handled.0.contains(&i.id)) {
        for (door_id, (door, pos)) in (&doors, &has_position).iter().with_id() {
            if bump.pos != pos.0 { continue; }
            if door.is_locked.is_none() { continue; }

            unlock_intents.push_back(UnlockIntent { id: bump.id, entity: bump.bumper, target: door_id });
        }
    }
}


pub fn on_bump_open_doors(mut map: UniqueViewMut<Map>,
                          has_position: View<HasPosition>,
                          mut doors: ViewMut<IsDoor>,
                          mut has_glyph: ViewMut<HasGlyph>,
                          mut bump_intents: UniqueViewMut<VecDeque<BumpIntent>>,
                          mut dirty: UniqueViewMut<IsDirty>,
                          mut handled: UniqueViewMut<ResolvedIntents>) {

    let mut resolved = Vec::new();
    for bump in bump_intents.iter().filter(|&&i| handled.not_handled(i)) {
        for (mut door, mut glyph, pos) in (&mut doors, &mut has_glyph, &has_position).iter().filter(|(d, _, p)| d.is_closed) {
            if pos.0 != bump.pos { continue; }
            if door.is_locked.is_some() { continue; }

            door.is_closed = false;
            glyph.0.ch = '_';
            glyph.0.fg = named_color(DARK_GRAY);
            let tile_index = map.point2d_to_index(pos.0);
            map.set_at_tile_index(tile_index, MapTile::Corridor);

            resolved.push(bump.id);
            dirty.0 = true;
        }
    }

    for id in resolved { handled.0.insert(id); }
}

pub fn on_unlock_if_has_key_for_door(items: View<IsItem>,
                                     mut carries: ViewMut<CarriesItem>,
                                     doors: ViewMut<IsDoor>,
                                     mut unlock_intents: UniqueViewMut<VecDeque<UnlockIntent>>,
                                     mut unlock_directives: UniqueViewMut<VecDeque<UnlockDirective>>,
                                     mut notify_directives: UniqueViewMut<VecDeque<NotifyDirective>>,
                                     mut log: UniqueViewMut<NotificationLog>,
                                     mut handled: UniqueViewMut<ResolvedIntents>) {

    let mut destroy_key = None;
    let mut resolved = Vec::new();
    for unlock in unlock_intents.iter().filter(|&&i| handled.not_handled(i)) {
        let target_id = unlock.target;
        let owner_id = unlock.entity;

        if let Some((door_id, door)) = (&doors).iter().with_id()
            .filter(|(id, _)| target_id == *id).next() {

            if let Some(key_id) = door.is_locked {
                if let Some((carry_id, _)) = (&carries).iter().with_id()
                        .filter(|(id, c)| c.owner == owner_id && c.item == key_id).next() {

                    let key = (&items).get(key_id).unwrap();
                    log.write(format!("You unlocked the door with the {}", key.item));

                    unlock_directives.push_back(UnlockDirective(door_id));
                    resolved.push(unlock.id);
                    destroy_key = Some(carry_id);
                } else {
                    notify_directives.push_back(notify_if_alive(unlock.id, format!("The door marked '{}' is locked.", door.sign).as_str()));
                }
            }
        }
    }

    for id in resolved { handled.0.insert(id); }

    if destroy_key.is_some() {
        (&mut carries).remove(destroy_key.unwrap());
    }
}