use std::collections::VecDeque;
use bracket_color::prelude::ColorPair;
use bracket_lib::prelude::{Point, Algorithm2D, BTerm, RED, DARK_RED, DARK_GRAY, GOLD, DARK_GOLDENROD, CRIMSON, BLACK, WHITE, ORANGE, DARK_ORANGE, YELLOW, GREEN, DARK_GREEN};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Remove, SparseSet, Storage, UniqueView, UniqueViewMut, View, ViewMut};
use crate::colors::{ColorShifter, named_color};
use crate::entity::{HasFieldOfView, HasGlyph, HasPosition, IsDirty, IsPlayer};
use crate::game::{Batch, Store};
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::{CarriesItem, InvestigateIntent, IsItem, NotificationLog, ResolvedIntents, UnlockDirective};
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::map::Map;
use crate::MAP_CONSOLE_LAYER;
use crate::tiles::{MapTile, TileIndex};

pub struct IsDoor(pub bool);

#[derive(Clone, Eq, PartialEq)]
pub struct IsLocked {
    pub key: EntityId,
}
pub struct IsKnown;

pub fn render_doors(mut batch: UniqueViewMut<Batch>,
                    map: UniqueView<Map>,
                    doors: View<IsDoor>,
                    has_position: View<HasPosition>,
                    has_glyph: View<HasGlyph>,
                    has_fov: View<HasFieldOfView>,
                    is_player: View<IsPlayer>,) {

    if let Some((sight, _)) = (&has_fov, &is_player).iter().take(1).next() {
        batch.0.target(MAP_CONSOLE_LAYER);

        for (_, pos, glyph) in (&doors, &has_position, &has_glyph).iter() {
            let tile = map.point2d_to_index(Point::from(pos.0));
            let fg = if map.is_tile_visible(tile) { glyph.0.fg } else { named_color(DARK_GRAY) };

            if sight.fov.contains(&pos.0) {
                batch.0.set(pos.0, ColorPair::new(fg, named_color(BLACK)), glyph.0.ch as u16);
            }
        }
    }
}


pub fn render_locked_doors(mut batch: UniqueViewMut<Batch>,
                           doors: View<IsDoor>,
                           has_position: View<HasPosition>,
                           has_glyph: View<HasGlyph>,
                           is_locked: View<IsLocked>,
                           has_fov: View<HasFieldOfView>,
                           is_player: View<IsPlayer>,) {

    if let Some((sight, _)) = (&has_fov, &is_player).iter().take(1).next() {
        batch.0.target(MAP_CONSOLE_LAYER);

        for (_, pos, glyph, _) in (&doors, &has_position, &has_glyph, &is_locked).iter() {
            let is_visible = sight.fov.contains(&pos.0);
            let fg = if is_visible { named_color(CRIMSON) } else { named_color(DARK_RED) };

            if is_visible {
                batch.0.set(pos.0, ColorPair::new(fg, named_color(BLACK)), glyph.0.ch as u16);
            }
        }
    }
}

pub fn render_known_locked_doors(mut batch: UniqueViewMut<Batch>,
                                 doors: View<IsDoor>,
                                 has_position: View<HasPosition>,
                                 has_glyph: View<HasGlyph>,
                                 is_locked: View<IsLocked>,
                                 is_known: View<IsKnown>,
                                 carries: View<CarriesItem>,
                                 has_fov: View<HasFieldOfView>,
                                 is_player: View<IsPlayer>,) {

    if let Some((player_id, (sight, _))) = (&has_fov, &is_player).iter().with_id().take(1).next() {
        batch.0.target(MAP_CONSOLE_LAYER);

        for (_, pos, glyph, locked, _) in (&doors, &has_position, &has_glyph, &is_locked, &is_known).iter() {
            let is_visible = sight.fov.contains(&pos.0);
            let not_found = (&carries).iter().
                filter(|c| c.owner == player_id && c.item == locked.key)
                .collect::<Vec<_>>().is_empty();

            let mut fg = named_color(if not_found { GOLD } else { GREEN });
            if !is_visible {
                fg = named_color(if not_found { DARK_GOLDENROD } else { DARK_GREEN });
            }

            if is_visible {
                batch.0.set(pos.0, ColorPair::new(fg, named_color(BLACK)), glyph.0.ch as u16);
            }
        }
    }
}

pub fn on_bump_interpret_as_door_unlock_intent(doors: View<IsDoor>,
                                               locked: View<IsLocked>,
                                               has_position: View<HasPosition>,
                                               mut bump_intents: UniqueViewMut<VecDeque<BumpIntent>>,
                                               mut unlock_intents: UniqueViewMut<VecDeque<UnlockIntent>>,
                                               mut handled: UniqueViewMut<ResolvedIntents>) {

    for bump in bump_intents.iter().filter(|i| !handled.0.contains(&i.id)) {
        for (door_id, (_, _, pos)) in (&doors, &locked, &has_position).iter().with_id() {
            if bump.pos != pos.0 { continue; }

            unlock_intents.push_back(UnlockIntent { id: bump.id, entity: bump.bumper, target: door_id });
        }
    }
}


pub fn on_bump_open_doors(mut map: UniqueViewMut<Map>,
                          has_position: View<HasPosition>,
                          mut doors: ViewMut<IsDoor>,
                          mut locked: ViewMut<IsLocked>,
                          mut has_glyph: ViewMut<HasGlyph>,
                          mut bump_intents: UniqueViewMut<VecDeque<BumpIntent>>,
                          mut dirty: UniqueViewMut<IsDirty>,
                          mut handled: UniqueViewMut<ResolvedIntents>) {

    let mut resolved = Vec::new();
    for bump in bump_intents.iter().filter(|i| !handled.0.contains(&i.id)) {
        for (_, mut door, mut glyph, pos) in (!&locked, &mut doors, &mut has_glyph, &has_position).iter()
            .filter(|(_, d, _, p)| d.0) {

            if pos.0 != bump.pos { continue; }

            door.0 = false;
            glyph.0.set_ch('_');
            glyph.0.set_fg(named_color(DARK_GRAY));
            let tile_index = map.point2d_to_index(pos.0);
            map.set_at_tile_index(tile_index, MapTile::Corridor);

            resolved.push(bump.id);
            dirty.0 = true;
        }
    }

    for id in resolved { handled.0.insert(id); }
}


pub fn on_bump_interpret_as_investigate_intent(mut bump_intents: UniqueViewMut<VecDeque<BumpIntent>>,
                                               mut investigate_intents: UniqueViewMut<VecDeque<InvestigateIntent>>,
                                               mut handled: UniqueViewMut<ResolvedIntents>) {

    for bump in bump_intents.iter().filter(|i| !handled.0.contains(&i.id)) {
        investigate_intents.push_back(InvestigateIntent{ id: bump.id, pos: bump.pos });
    }
}


pub fn on_unlock_if_has_key_for_door(items: View<IsItem>,
                                     is_locked: View<IsLocked>,
                                     carries: View<CarriesItem>,
                                     mut unlock_intents: UniqueViewMut<VecDeque<UnlockIntent>>,
                                     mut unlock_directives: UniqueViewMut<VecDeque<UnlockDirective>>,
                                     mut log: UniqueViewMut<NotificationLog>,
                                     mut handled: UniqueViewMut<ResolvedIntents>) {

    let mut resolved = Vec::new();
    for unlock in unlock_intents.iter().filter(|i| !handled.0.contains(&i.id)) {
        let target_id = unlock.target;
        let owner_id = unlock.entity;

        for (lock_id, lock) in (&is_locked).iter().with_id()
            .filter(|(id, _)| target_id == *id) {

            let key_id = lock.key;

            for _ in (&carries).iter()
                .filter(|c| c.owner == owner_id && c.item == key_id) {

                let key = (&items).get(lock.key).unwrap();
                log.write(format!("You unlocked the door with {}", key.item));

                unlock_directives.push_back(UnlockDirective(lock_id));
                resolved.push(unlock.id);
            }
        }
    }

    for id in resolved { handled.0.insert(id); }
}

pub fn on_investigate_lock(has_pos: View<HasPosition>,
                           is_locked: View<IsLocked>,
                           is_item: View<IsItem>,
                           mut known: ViewMut<IsKnown>,
                           mut investigate_intents: UniqueViewMut<VecDeque<InvestigateIntent>>,
                           mut log: UniqueViewMut<NotificationLog>,
                           mut dirty: UniqueViewMut<IsDirty>,
                           mut handled: UniqueViewMut<ResolvedIntents>) {

    for investigation in investigate_intents.iter().filter(|i| !handled.0.contains(&i.id)) {
        for (id, (pos, lock)) in (&has_pos, &is_locked).iter().with_id() {
            if pos.0 != investigation.pos { continue; }

            if let Ok(key) = (&is_item).get(lock.key) {
                log.write(format!("You need the {} to open this lock.", key.item));
                dirty.0 = true;

                known.add_entity(id, IsKnown);
            }
        }
    }
}