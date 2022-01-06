use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::ops::Index;
use bracket_color::prelude::{BLACK, DARK_GRAY};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Not, Remove, Storage, UniqueView, UniqueViewMut, View, ViewMut};
use crate::colors::{ColorShifter, named_color};
use crate::core_systems::IsCharacter;
use crate::entity::{HasGlyph, HasPosition, IsDirty, IsInvisible};
use crate::game::Store;
use crate::game_systems::{BumpIntent, CollectIntent, Handle, MoveDirective, NotificationLog};
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

pub fn on_bump_interpret_as_collect_item_intent(items: View<IsItem>,
                                                has_position: View<HasPosition>,
                                                mut bump_intents: ViewMut<Handle<BumpIntent>>,
                                                mut collect_intents: ViewMut<Handle<CollectIntent>>,
                                                mut entities: EntitiesViewMut,) {

    for mut bump in (&mut bump_intents).iter().
        filter(|b| !b.handled) {

        for (item_id, (_, pos)) in (&items, &has_position).iter().with_id()
            .filter(|(_, (i, _))| !i.is_collected) {

            if pos.0 != bump.intent.pos { continue; }

            entities.add_entity((&mut collect_intents, ),
                (Handle::new(CollectIntent{ collector: bump.intent.bumper, item: item_id }), ));

            bump.handled = true;
        }
    }
}


pub fn on_collect_default(mut items: ViewMut<IsItem>,
                          mut has_position: ViewMut<HasPosition>,
                          mut carries_item: ViewMut<CarriesItem>,
                          mut collect_intents: ViewMut<Handle<CollectIntent>>,
                          mut log: UniqueViewMut<NotificationLog>,
                          mut is_dirty: UniqueViewMut<IsDirty>,
                          mut entities: EntitiesViewMut,) {

    for mut collect in (&mut collect_intents).iter().
        filter(|c| !c.handled) {

        let mut item = (&mut items).get(collect.intent.item).unwrap();

        has_position.remove(collect.intent.item);
        item.is_collected = true;

        log.write(format!("Collected {}.", item.item));

        let carry = CarriesItem{ owner: collect.intent.collector, item: collect.intent.item };
        entities.add_entity((&mut carries_item,), (carry,));

        is_dirty.0 = true;
        collect.handled = true;
    }
}
