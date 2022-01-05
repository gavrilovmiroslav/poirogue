use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm, RED, DARK_RED, DARK_GRAY};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Remove, UniqueViewMut, View, ViewMut};
use crate::colors::named_color;
use crate::entity::{HasGlyph, HasPosition, IsDirty};
use crate::game::Store;
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::{CarriesItem, HasInventory, IsItem, NotificationLog};
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::map::Map;
use crate::store_helpers::StoreHelpers;
use crate::tiles::{MapTile, TileIndex};

pub struct IsDoor(pub bool);

#[derive(Clone, Eq, PartialEq)]
pub enum ItemSpendMode { Consume, Retain, }

pub struct IsLocked(pub EntityId, pub ItemSpendMode);

pub fn render_doors((map, ctx): (&mut Map, &mut BTerm),
                    doors: View<IsDoor>,
                    has_position: View<HasPosition>,
                    has_glyph: View<HasGlyph>,
                    is_locked: View<IsLocked>) {

    for (_, pos, glyph, _) in (&doors, &has_position, &has_glyph, !&is_locked).iter() {
        let tile = map.point2d_to_index(Point::from(pos.0));
        let fg = if map.is_tile_visible(tile) { glyph.0.fg } else { named_color(DARK_GRAY) };

        if map.is_tile_revealed(tile) {
            ctx.print_color(pos.0.x, pos.0.y, fg, named_color(BLACK),  glyph.0.ch);
        }
    }

    for (_, pos, glyph, _) in (&doors, &has_position, &has_glyph, &is_locked).iter() {
        let tile = map.point2d_to_index(Point::from(pos.0));
        let fg = if map.is_tile_visible(tile) { named_color(RED) } else { named_color(DARK_RED) };

        if map.is_tile_revealed(tile) {
            ctx.print_color(pos.0.x, pos.0.y, fg, named_color(BLACK), glyph.0.ch);
        }
    }
}


pub fn bump__interpret_as_door_unlock_intent(doors: View<IsDoor>,
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
    }
}


pub fn bump__open_doors(map: &mut Map,
                        has_position: View<HasPosition>,
                        mut doors: ViewMut<IsDoor>,
                        mut locked: ViewMut<IsLocked>,
                        mut has_glyph: ViewMut<HasGlyph>,
                        mut bump_intents: ViewMut<BumpIntent>,
                        mut dirty: ViewMut<IsDirty>) {

    let mut handled = Vec::new();
    for (bump_id, bump) in (&bump_intents).iter().with_id() {
        let index = map.point2d_to_index(bump.pos);

        for (id, (_, mut door, mut glyph, has_pos)) in (!&locked, &mut doors, &mut has_glyph, &has_position).iter().with_id()
            .filter(|(_, (_, d, _, p))| d.0 && p.0 == bump.pos) {

            door.0 = false;
            glyph.0.ch = '_';
            glyph.0.fg = named_color(DARK_GRAY);
            map.set_at_tile_index(index, MapTile::Corridor);

            dirty.add_entity(bump.bumper, IsDirty);
            handled.push(bump_id);
        }
    }

    for id in handled {
        bump_intents.remove(id);
    }
}


pub fn unlock__if_has_key_for_door(mut carries: ViewMut<CarriesItem>,
                                   mut is_locked: ViewMut<IsLocked>,
                                   mut unlock_intents: ViewMut<UnlockIntent>,) {

    let mut handled = Vec::new();

    for (unlock_intent_id, unlock) in (&unlock_intents).iter().with_id() {
        let owner_id = unlock.entity;

        for (lock_id, mut lock) in (&mut is_locked).iter().with_id()
            .filter(|(id, _)| unlock.target == *id) {

            let key_id = lock.0;

            for (carry_id, _) in (&carries).iter().with_id()
                .filter(|(_, c)| c.owner == owner_id && c.item == key_id) {

                handled.push((unlock_intent_id, carry_id, lock_id, lock.1.clone()));
            }
        }
    }

    for (unlock_id, carry_id, lock_id, key_spent) in handled {
        unlock_intents.remove(unlock_id);

        if key_spent == ItemSpendMode::Consume { // TODO: separate component?
            carries.remove(carry_id);
        }

        is_locked.remove(lock_id);
    }
}


pub fn unlock__default(mut unlock_intents: ViewMut<UnlockIntent>,
                       is_locked: View<IsLocked>,
                       is_item: View<IsItem>,
                       mut log: UniqueViewMut<NotificationLog>,) {

    for unlock_intent in (&unlock_intents).iter() {
        let lock = (&is_locked).get(unlock_intent.target).unwrap();
        let item = (&is_item).get(lock.0).unwrap();

        log.write(format!("This door is locked. It requires {} to open.", item.item.clone()));
    }

    unlock_intents.clear();
}