use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::ops::Index;
use bracket_color::prelude::{BLACK, DARK_GRAY};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Not, Remove, UniqueView, UniqueViewMut, View, ViewMut};
use crate::colors::{ColorShifter, named_color};
use crate::core_systems::IsCharacter;
use crate::entity::{HasGlyph, HasPosition, IsDirty, IsInvisible};
use crate::game::Store;
use crate::game_systems::{BumpIntent, CollectIntent, MoveDirective, NotificationLog};
use crate::map::Map;
use crate::tiles::{MapTile, TileIndex};

pub type Item = String;

pub struct IsItem {
    pub item: Item,
    pub is_collected: bool,
}

pub struct CarriesItem {
    pub owner: EntityId,
    pub item: EntityId,
}

pub fn render_items((map, ctx): (&Map, &mut BTerm),
                    has_position: View<HasPosition>,
                    has_glyph: View<HasGlyph>,
                    is_invisible: View<IsInvisible>,
                    is_item: View<IsItem>,
                    is_dirty: UniqueView<IsDirty>, ) {

    if is_dirty.0 {
        for (_, pos, glyph, _) in (&is_item, &has_position, &has_glyph, !&is_invisible).iter()
            .filter(|(i, _, _, _)| !i.is_collected) {
            let tile = map.point2d_to_index(Point::from(pos.0));
            let fg = if map.is_tile_visible(tile) { glyph.0.fg } else { named_color(DARK_GRAY).darken(0.5) };

            if map.is_tile_revealed(tile) {
                ctx.print_color(pos.0.x, pos.0.y, fg, named_color(BLACK), glyph.0.ch);
            }
        }
    }
}

pub fn on_bump_interpret_as_collect_item_intent(characters: View<IsCharacter>,
                                              items: View<IsItem>,
                                              has_position: View<HasPosition>,
                                              is_invisible: View<IsInvisible>,
                                              mut bump_intents: ViewMut<BumpIntent>,
                                              mut collect_intents: ViewMut<CollectIntent>,
                                              mut entities: EntitiesViewMut,) {

    let mut handled = Vec::new();
    for (bump_id, bump) in (&bump_intents).iter().with_id() {
        for (item_id, (_, _, _)) in (&items, &has_position, !&is_invisible).iter().with_id()
            .filter(|(_, (i, p, _))| !i.is_collected && p.0 == bump.pos) {

            entities.add_entity((&mut collect_intents, ), (CollectIntent{ collector: bump.bumper, item: item_id }, ));
            handled.push(bump_id);
        }
    }

    for id in handled {
        bump_intents.remove(id);
    }
}


pub fn collect__default(mut items: ViewMut<IsItem>,
                        mut has_position: ViewMut<HasPosition>,
                        mut carries_item: ViewMut<CarriesItem>,
                        mut collect_intents: ViewMut<CollectIntent>,
                        mut entities: EntitiesViewMut,
                        mut log: UniqueViewMut<NotificationLog>,
                        mut is_dirty: UniqueViewMut<IsDirty>) {

    for collect_intent in (&collect_intents).iter() {
        let mut item = (&mut items).get(collect_intent.item).unwrap();

        has_position.remove(collect_intent.item);
        item.is_collected = true;

        log.write(format!("Collected {}.", item.item));

        let carry = CarriesItem{ owner: collect_intent.collector, item: collect_intent.item };
        entities.add_entity((&mut carries_item,), (carry,));

        is_dirty.0 = true;
    }

    collect_intents.clear();
}
