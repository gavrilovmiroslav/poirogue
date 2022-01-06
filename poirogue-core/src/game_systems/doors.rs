use bracket_lib::prelude::{Point, Algorithm2D, BTerm, RED, DARK_RED, DARK_GRAY, GOLD, DARK_GOLDENROD, CRIMSON, BLACK, WHITE, ORANGE, DARK_ORANGE, YELLOW, GREEN, DARK_GREEN};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Remove, Storage, UniqueView, UniqueViewMut, View, ViewMut};
use crate::colors::{ColorShifter, named_color};
use crate::entity::{HasGlyph, HasPosition, IsDirty, IsPlayer};
use crate::game::Store;
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::{CarriesItem, InvestigateIntent, IsItem, NotificationLog};
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::map::Map;
use crate::store_helpers::StoreHelpers;
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
            let tile = map.point2d_to_index(Point::from(pos.0));

            let (player_id, _) = (&is_player).iter().with_id().take(1).collect::<Vec<(EntityId, _)>>()[0];
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
                                             mut bump_intents: ViewMut<BumpIntent>,
                                             mut unlock_intents: ViewMut<UnlockIntent>,
                                             mut entities: EntitiesViewMut,) {

    let mut handled = Vec::new();
    for (bump_id, bump) in (&bump_intents).iter().with_id() {
        let pt = Point::from(bump.pos);

        for (door_id, (door, _, _)) in (&doors, &locked, &has_position).iter().with_id()
            .filter(|(_, (_, _, p))| p.0 == pt) {

            entities.add_entity((&mut unlock_intents, ), (UnlockIntent { entity: bump.bumper, target: door_id }, ));
            handled.push(bump_id);
        }
    }

    for id in handled {
        bump_intents.remove(id);
        entities.delete(id);
    }
}


pub fn on_bump_open_doors(map: &mut Map,
                          has_position: View<HasPosition>,
                          mut doors: ViewMut<IsDoor>,
                          mut locked: ViewMut<IsLocked>,
                          mut has_glyph: ViewMut<HasGlyph>,
                          mut bump_intents: ViewMut<BumpIntent>,
                          mut dirty: UniqueViewMut<IsDirty>,
                          mut entities: EntitiesViewMut,) {

    let mut handled = Vec::new();
    for (bump_id, bump) in (&bump_intents).iter().with_id() {
        let index = map.point2d_to_index(bump.pos);

        for (id, (_, mut door, mut glyph, has_pos)) in (!&locked, &mut doors, &mut has_glyph, &has_position).iter().with_id()
            .filter(|(_, (_, d, _, p))| d.0 && p.0 == bump.pos) {

            door.0 = false;
            glyph.0.ch = '_';
            glyph.0.fg = named_color(DARK_GRAY);
            map.set_at_tile_index(index, MapTile::Corridor);

            dirty.0 = true;
            handled.push(bump_id);
        }
    }

    for id in handled {
        bump_intents.remove(id);
        entities.delete(id);
    }
}


pub fn on_unlock_if_has_key_for_door(items: View<IsItem>,
                                     mut carries: ViewMut<CarriesItem>,
                                     mut is_locked: ViewMut<IsLocked>,
                                     mut is_known: ViewMut<IsKnown>,
                                     mut lock_spends_key: ViewMut<ObjectUsedUp>,
                                     mut unlock_intents: ViewMut<UnlockIntent>,
                                     mut log: UniqueViewMut<NotificationLog>,
                                     mut dirty: UniqueViewMut<IsDirty>,
                                     mut entities: EntitiesViewMut) {

    let mut handled = Vec::new();

    for (unlock_intent_id, unlock) in (&unlock_intents).iter().with_id() {
        let owner_id = unlock.entity;

        for (lock_id, mut lock) in (&mut is_locked).iter().with_id()
            .filter(|(id, _)| unlock.target == *id) {

            let key_id = lock.key;
            let key = (&items).get(lock.key).unwrap();

            let mut has_key = false;
            for (carry_id, _) in (&carries).iter().with_id()
                .filter(|(_, c)| c.owner == owner_id && c.item == key_id) {

                handled.push((unlock_intent_id, carry_id, lock_id, key));
                has_key = true;
            }

            if !has_key {
                is_known.add_entity(lock_id, IsKnown);
            }
        }
    }

    if !handled.is_empty() {
        dirty.0 = true;
    }

    for (unlock_intent_id, carry_id, lock_id, key) in handled {
        unlock_intents.remove(unlock_intent_id);
        entities.delete(unlock_intent_id);

        let mut key_spent_message = ".";

        let key_spent = (&lock_spends_key).get(lock_id).is_ok();

        if key_spent {
            key_spent_message = ". The key disintegrates instantly.";
            carries.remove(carry_id);
        }

        is_locked.remove(lock_id);
        lock_spends_key.remove(lock_id);
        log.write(format!("You unlocked the door with {}{}", key.item, key_spent_message));
    }
}


pub fn on_unlock_default(mut unlock_intents: ViewMut<UnlockIntent>,
                         mut investigate_intents: ViewMut<InvestigateIntent>,
                         mut entities: EntitiesViewMut,) {

    for (unlock_intent_id, unlock_intent) in (&unlock_intents).iter().with_id() {
        entities.add_entity((&mut investigate_intents,), (InvestigateIntent{ entity: unlock_intent.target },));
        entities.delete(unlock_intent_id);
    }

    unlock_intents.clear();
}

pub fn on_investigate_lock(mut investigate_intents: ViewMut<InvestigateIntent>,
                           is_locked: View<IsLocked>,
                           is_item: View<IsItem>,
                           mut log: UniqueViewMut<NotificationLog>,
                           mut dirty: UniqueViewMut<IsDirty>,
                           mut entities: EntitiesViewMut,) {

    let mut handled = Vec::new();

    for (id, investigation) in (&investigate_intents).iter().with_id() {
        if let Ok(lock) = (&is_locked).get(investigation.entity) {
            if let Ok(key) = (&is_item).get(lock.key) {
                log.write(format!("You need the {} to open this lock.", key.item));
                handled.push(id);
            }
        }
    }

    if !handled.is_empty() {
        dirty.0 = true;
    }

    for id in handled {
        investigate_intents.remove(id);
        entities.delete(id);
    }
}

pub fn on_investigate_default(mut investigate_intents: ViewMut<InvestigateIntent>,
                              mut entities: EntitiesViewMut) {
    for (id, _) in (&investigate_intents).iter().with_id() {
        entities.delete(id);
    }

    investigate_intents.clear();
}