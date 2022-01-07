use bracket_lib::prelude::{Point, Algorithm2D, BTerm, RED, DARK_RED, DARK_GRAY, GOLD, DARK_GOLDENROD, CRIMSON, BLACK, WHITE, ORANGE, DARK_ORANGE, YELLOW, GREEN, DARK_GREEN};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Remove, Storage, UniqueView, UniqueViewMut, View, ViewMut};
use crate::colors::{ColorShifter, named_color};
use crate::entity::{HasGlyph, HasPosition, IsDirty, IsPlayer};
use crate::game::Store;
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::{CarriesItem, Handle, InvestigateIntent, IsItem, NotificationLog, UnlockDirective};
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::map::Map;
use crate::tiles::{MapTile, TileIndex};

pub struct IsDoor(pub bool);

#[derive(Clone, Eq, PartialEq)]
pub struct IsLocked {
    pub key: EntityId,
}
pub struct ObjectUsedUp;
pub struct IsKnown;

pub fn render_doors((map, ctx): (&Map, &mut BTerm),
                    doors: View<IsDoor>,
                    has_position: View<HasPosition>,
                    has_glyph: View<HasGlyph>,
                    is_dirty: UniqueView<IsDirty>, ) {

    if is_dirty.0 {
        ctx.set_active_console(0);

        for (_, pos, glyph) in (&doors, &has_position, &has_glyph).iter() {
            let tile = map.point2d_to_index(Point::from(pos.0));
            let fg = if map.is_tile_visible(tile) { glyph.0.fg } else { named_color(DARK_GRAY) };

            if map.is_tile_revealed(tile) {
                ctx.print_color(pos.0.x, pos.0.y, fg, named_color(BLACK), glyph.0.ch);
            }
        }
    }
}


pub fn render_locked_doors((map, ctx): (&Map, &mut BTerm),
                           doors: View<IsDoor>,
                           has_position: View<HasPosition>,
                           has_glyph: View<HasGlyph>,
                           is_locked: View<IsLocked>,
                           is_dirty: UniqueView<IsDirty>, ) {

    if is_dirty.0 {
        for (_, pos, glyph, _) in (&doors, &has_position, &has_glyph, &is_locked).iter() {
            let tile = map.point2d_to_index(Point::from(pos.0));

            let fg = if map.is_tile_visible(tile) { named_color(CRIMSON) } else { named_color(DARK_RED) };

            if map.is_tile_revealed(tile) {
                ctx.print_color(pos.0.x, pos.0.y, fg, named_color(BLACK), glyph.0.ch);
            }
        }
    }
}


pub fn render_known_locked_doors((map, ctx): (&Map, &mut BTerm),
                           doors: View<IsDoor>,
                           has_position: View<HasPosition>,
                           has_glyph: View<HasGlyph>,
                           is_locked: View<IsLocked>,
                           is_player: View<IsPlayer>,
                           is_known: View<IsKnown>,
                           carries: View<CarriesItem>,
                           is_dirty: UniqueView<IsDirty>, ) {

    if is_dirty.0 {
        for (_, pos, glyph, locked, _) in (&doors, &has_position, &has_glyph, &is_locked, &is_known).iter() {
            let tile = map.point2d_to_index(pos.0);

            let (player_id, _) = (&is_player).iter().with_id().collect::<Vec<(EntityId, _)>>().first().unwrap().clone();

            let not_found = (&carries).iter().
                filter(|c| c.owner == player_id && c.item == locked.key)
                .collect::<Vec<_>>().is_empty();

            let mut fg = named_color(if not_found { GOLD } else { GREEN });
            if !map.is_tile_visible(tile) {
                fg = named_color(if not_found { DARK_GOLDENROD } else { DARK_GREEN });
            }

            if map.is_tile_revealed(tile) {
                ctx.print_color(pos.0.x, pos.0.y, fg, named_color(BLACK), glyph.0.ch);
            }
        }
    }
}


pub fn on_bump_interpret_as_door_unlock_intent(doors: View<IsDoor>,
                                             locked: View<IsLocked>,
                                             has_position: View<HasPosition>,
                                             mut bump_intents: ViewMut<Handle<BumpIntent>>,
                                             mut unlock_intents: ViewMut<Handle<UnlockIntent>>,
                                             mut entities: EntitiesViewMut,) {

    for mut bump in (&mut bump_intents).iter()
        .filter(|b| !b.handled) {

        for (door_id, (_, _, pos)) in (&doors, &locked, &has_position).iter().with_id() {
            if bump.intent.pos != pos.0 { continue; }

            entities.add_entity((&mut unlock_intents, ), (Handle::new(UnlockIntent { entity: bump.intent.bumper, target: door_id }), ));
            bump.handled = true;
        }
    }
}


pub fn on_bump_open_doors(map: &mut Map,
                          has_position: View<HasPosition>,
                          mut doors: ViewMut<IsDoor>,
                          mut locked: ViewMut<IsLocked>,
                          mut has_glyph: ViewMut<HasGlyph>,
                          mut bump_intents: ViewMut<Handle<BumpIntent>>,
                          mut dirty: UniqueViewMut<IsDirty>, ) {

    for mut bump in (&mut bump_intents).iter()
        .filter(|b| !b.handled) {

        for (_, mut door, mut glyph, pos) in (!&locked, &mut doors, &mut has_glyph, &has_position).iter()
            .filter(|(_, d, _, p)| d.0) {

            if pos.0 != bump.intent.pos { continue; }

            door.0 = false;
            glyph.0.ch = '_';
            glyph.0.fg = named_color(DARK_GRAY);
            map.set_at_tile_index(map.point2d_to_index(pos.0), MapTile::Corridor);

            dirty.0 = true;
            bump.handled = true;
        }
    }
}


pub fn on_unlock_if_has_key_for_door(items: View<IsItem>,
                                     is_locked: View<IsLocked>,
                                     carries: View<CarriesItem>,
                                     mut unlock_intents: ViewMut<Handle<UnlockIntent>>,
                                     mut unlock_directives: ViewMut<UnlockDirective>,
                                     mut log: UniqueViewMut<NotificationLog>,
                                     mut entities: EntitiesViewMut) {

    for mut unlock in (&mut unlock_intents).iter()
        .filter(|u| !u.handled) {

        let target_id = unlock.intent.target;
        let owner_id = unlock.intent.entity;

        for (lock_id, lock) in (&is_locked).iter().with_id()
            .filter(|(id, _)| target_id == *id) {

            let key_id = lock.key;

            for _ in (&carries).iter()
                .filter(|c| c.owner == owner_id && c.item == key_id) {

                let key = (&items).get(lock.key).unwrap();
                log.write(format!("You unlocked the door with {}", key.item));

                entities.add_entity((&mut unlock_directives,), (UnlockDirective(lock_id),));
                unlock.handled = true;
            }
        }
    }
}


pub fn on_unlock_default(mut unlock_intents: ViewMut<Handle<UnlockIntent>>,
                         mut investigate_intents: ViewMut<Handle<InvestigateIntent>>,
                         mut entities: EntitiesViewMut,) {

    for mut unlock_intent in (&mut unlock_intents).iter()
        .filter(|u| !u.handled) {

        entities.add_entity((&mut investigate_intents,), (Handle::new(InvestigateIntent{ entity:  unlock_intent.intent.target }),));
        unlock_intent.handled = true;
    }
}

pub fn on_investigate_lock(mut investigate_intents: ViewMut<Handle<InvestigateIntent>>,
                           is_locked: View<IsLocked>,
                           is_item: View<IsItem>,
                           mut known: ViewMut<IsKnown>,
                           mut log: UniqueViewMut<NotificationLog>,
                           mut dirty: UniqueViewMut<IsDirty>,) {

    for mut investigation in (&mut investigate_intents).iter()
        .filter(|i| !i.handled) {

        if let Ok(lock) = (&is_locked).get(investigation.intent.entity) {
            if let Ok(key) = (&is_item).get(lock.key) {
                log.write(format!("You need the {} to open this lock.", key.item));
                dirty.0 = true;

                known.add_entity(investigation.intent.entity, IsKnown);
                investigation.handled = true;
            }
        }
    }
}

pub fn on_investigate_default(mut investigate_intents: ViewMut<Handle<InvestigateIntent>>,) {

    for mut investigation in (&mut investigate_intents).iter() {
        investigation.handled = true;
    }
}