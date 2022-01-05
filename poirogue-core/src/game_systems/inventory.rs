use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::ops::Index;
use bracket_color::prelude::{BLACK, DARK_GRAY};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Not, Remove, UniqueViewMut, View, ViewMut};
use crate::colors::{ColorShifter, named_color};
use crate::core_systems::IsCharacter;
use crate::entity::{HasGlyph, HasPosition, IsInvisible};
use crate::game::Store;
use crate::game_systems::{BumpIntent, CollectIntent, MoveDirective, NotificationLog};
use crate::map::Map;
use crate::tiles::{MapTile, TileIndex};

pub type Item = String;
pub struct HasInventory(pub HashSet<Item>);

pub struct IsItem {
    pub item: Item,
    pub is_collected: bool,
}

pub struct CarriesItem {
    pub owner: EntityId,
    pub item: EntityId,
}

pub fn render_items((map, ctx): (&mut Map, &mut BTerm),
                    has_position: View<HasPosition>,
                    has_glyph: View<HasGlyph>,
                    is_invisible: View<IsInvisible>,
                    is_item: View<IsItem>,) {

    for (_, pos, glyph, _) in (&is_item, &has_position, &has_glyph, !&is_invisible).iter()
        .filter(|(i, _, _, _)| !i.is_collected) {

        let tile = map.point2d_to_index(Point::from(pos.0));
        let fg = if map.is_tile_visible(tile) { glyph.0.fg } else { named_color(DARK_GRAY).darken(0.5) };

        if map.is_tile_revealed(tile) {
            ctx.print_color(pos.0.x, pos.0.y, fg, named_color(BLACK),  glyph.0.ch);
        }
    }
}

pub fn bump__interpret_as_collect_item_intent(characters: View<IsCharacter>,
                                              items: View<IsItem>,
                                              has_position: View<HasPosition>,
                                              has_inventory: View<HasInventory>,
                                              is_invisible: View<IsInvisible>,
                                              mut bump_intents: ViewMut<BumpIntent>,
                                              mut collect_intents: ViewMut<CollectIntent>,
                                              mut entities: EntitiesViewMut,) {

    let mut handled = Vec::new();
    for (bump_id, bump) in (&bump_intents).iter().with_id() {
        for (_, (_, inventory)) in (&characters, &has_inventory).iter().with_id()
            .filter(|(id, _)| *id == bump.bumper) {

            for (item_id, (_, _, _)) in (&items, &has_position, !&is_invisible).iter().with_id()
                .filter(|(_, (i, p, _))| !i.is_collected && p.0 == bump.pos) {

                entities.add_entity((&mut collect_intents, ), (CollectIntent{ collector: bump.bumper, item: item_id }, ));
                handled.push(bump_id);
            }
        }
    }

    for id in handled {
        bump_intents.remove(id);
    }
}


pub fn collect__default(mut items: ViewMut<IsItem>,
                        mut has_position: ViewMut<HasPosition>,
                        mut carries_item: ViewMut<CarriesItem>,
                        mut has_inventory: ViewMut<HasInventory>,
                        mut collect_intents: ViewMut<CollectIntent>,
                        mut entities: EntitiesViewMut,
                        mut log: UniqueViewMut<NotificationLog>) {

    for collect_intent in (&collect_intents).iter() {
        let mut inv = (&mut has_inventory).get(collect_intent.collector).unwrap();
        let mut item = (&mut items).get(collect_intent.item).unwrap();

        inv.0.insert(item.item.clone());
        has_position.remove(collect_intent.item);
        item.is_collected = true;

        log.write(format!("Collected {}.", item.item.clone()));

        let carry = CarriesItem{ owner: collect_intent.collector, item: collect_intent.item };
        entities.add_entity((&mut carries_item,), (carry,));
    }

    collect_intents.clear();
}
