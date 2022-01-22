use std::borrow::BorrowMut;
use std::collections::{HashSet, VecDeque};
use std::ops::Index;
use bracket_color::prelude::{BLACK, ColorPair, DARK_GRAY};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Not, Remove, SparseSet, Storage, UniqueView, UniqueViewMut, View, ViewMut};
use crate::colors::{ColorShifter, named_color};
use crate::core_systems::IsCharacter;
use crate::entity::{HasFieldOfView, HasGlyph, HasPosition, IsDirty, IsInvisible, IsPlayer};
use crate::game::{Batch, Store};
use crate::game_systems::{BumpIntent, CollectIntent, MoveDirective, NotificationLog, ResolvedIntents};
use crate::map::Map;
use crate::MAP_CONSOLE_LAYER;
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

pub fn render_items(mut batch: UniqueViewMut<Batch>,
                    has_position: View<HasPosition>,
                    has_glyph: View<HasGlyph>,
                    is_item: View<IsItem>,
                    has_fov: View<HasFieldOfView>,
                    is_player: View<IsPlayer>,) {

    if let Some((fov, _)) = (&has_fov, &is_player).iter().take(1).next() {
        batch.0.target(MAP_CONSOLE_LAYER);

        for (_, pos, glyph) in (&is_item, &has_position, &has_glyph).iter().filter(|i| !i.0.is_collected) {
            let is_visible = fov.0.contains(&pos.0);
            let fg = if is_visible { glyph.0.fg } else { named_color(DARK_GRAY).darken(0.5) };

            if is_visible {
                batch.0.set(pos.0, ColorPair::new(fg, named_color(BLACK)), glyph.0.ch as u16);
            }
        }
    }
}

pub fn on_bump_interpret_as_collect_item_intent(items: View<IsItem>,
                                                has_position: View<HasPosition>,
                                                mut bump_intents: UniqueViewMut<VecDeque<BumpIntent>>,
                                                mut collect_intents: UniqueViewMut<VecDeque<CollectIntent>>,
                                                mut handled: UniqueViewMut<ResolvedIntents>) {

    for bump in bump_intents.iter().filter(|i| !handled.0.contains(&i.id)) {
        for (item_id, (_, pos)) in (&items, &has_position).iter().with_id()
            .filter(|(_, (i, _))| !i.is_collected) {

            if pos.0 == bump.pos {
                collect_intents.push_back(CollectIntent { id: bump.id, collector: bump.bumper, item: item_id });
            }
        }
    }
}


pub fn on_collect_if_possible(mut items: ViewMut<IsItem>,
                              mut has_position: ViewMut<HasPosition>,
                              mut carries_item: ViewMut<CarriesItem>,
                              mut collect_intents: UniqueViewMut<VecDeque<CollectIntent>>,
                              mut log: UniqueViewMut<NotificationLog>,
                              mut is_dirty: UniqueViewMut<IsDirty>,
                              mut entities: EntitiesViewMut,
                              mut handled: UniqueViewMut<ResolvedIntents>) {

    let mut resolved = Vec::new();
    for collect in collect_intents.iter().filter(|i| !handled.0.contains(&i.id)) {
        if let Ok(mut item) = (&mut items).get(collect.item)
        {
            has_position.remove(collect.item);
            item.is_collected = true;

            log.write(format!("Collected {}.", item.item));

            let carry = CarriesItem { owner: collect.collector, item: collect.item };
            entities.add_entity((&mut carries_item, ), (carry, ));

            resolved.push(collect.id);
            is_dirty.0 = true;
        }
    }

    for id in resolved { handled.0.insert(id); }
}
