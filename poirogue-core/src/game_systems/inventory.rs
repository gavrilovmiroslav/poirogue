use std::borrow::BorrowMut;
use std::collections::{HashSet, VecDeque};
use std::ops::Index;
use bracket_color::prelude::{BLACK, ColorPair, DARK_GRAY};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm, VirtualKeyCode, Rect, WHITE, GRAY};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Not, Remove, SparseSet, Storage, UniqueView, UniqueViewMut, View, ViewMut};
use crate::colors::{ColorShifter, named_color};
use crate::commands::GameplayContext;
use crate::entity::{HasSight, HasGlyph, HasPosition, IsDirty, Player};
use crate::game::{Batch, Store};
use crate::game_systems::{BumpIntent, CollectIntent, MoveDirective, NotificationLog, ResolvedIntents};
use crate::input::{InputSnapshot, KeyboardSnapshot};
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
                    has_sight: View<HasSight>,
                    player: UniqueView<Player>,) {

    if let Some(entity) = player.entity {
        if let Ok((sight,)) = (&has_sight,).get(entity) {
            batch.0.target(MAP_CONSOLE_LAYER);

            for (_, pos, glyph) in (&is_item, &has_position, &has_glyph).iter().filter(|i| !i.0.is_collected) {
                let is_visible = sight.field_of_view.contains(&pos.0);
                let fg = if is_visible { glyph.0.fg } else { named_color(DARK_GRAY).darken(0.5) };

                if is_visible {
                    batch.0.set(pos.0, ColorPair::new(fg, named_color(BLACK)), glyph.0.ch as u16);
                }
            }
        }
    }
}

pub fn on_bump_interpret_as_collect_item_intent(items: View<IsItem>,
                                                has_position: View<HasPosition>,
                                                mut bump_intents: UniqueViewMut<VecDeque<BumpIntent>>,
                                                mut collect_intents: UniqueViewMut<VecDeque<CollectIntent>>,
                                                mut handled: UniqueViewMut<ResolvedIntents>) {

    for bump in bump_intents.iter().filter(|&&i| handled.not_handled(i)) {
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
    for collect in collect_intents.iter().filter(|&&i| handled.not_handled(i)) {
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


pub fn interpret_player_input_as_inventory_access(mut keyboard: UniqueViewMut<KeyboardSnapshot>,
                                                  mut dirty: UniqueViewMut<IsDirty>,
                                                  mut context: UniqueViewMut<GameplayContext>,) {

    match *context {
        GameplayContext::MainGame => {
            if keyboard.is_pressed(VirtualKeyCode::I) {
                *context = GameplayContext::Inventory;
                dirty.0 = true;
            }
        },
        GameplayContext::Inventory => {
            if keyboard.is_pressed(VirtualKeyCode::Escape) {
                keyboard.consume(VirtualKeyCode::Escape);
                *context = GameplayContext::MainGame;
                dirty.0 = true;
            }
        }
    }
}

pub fn render_inventory(mut batch: UniqueViewMut<Batch>,
                        is_item: View<IsItem>,
                        is_player: View<Player>,
                        carries: View<CarriesItem>,
                        context: UniqueView<GameplayContext>) {

    if *context == GameplayContext::Inventory {
        batch.0.target(MAP_CONSOLE_LAYER);
        if let Some((player_id, _)) = (&is_player).iter().with_id().next() {
            batch.0.draw_box(Rect::with_size(40, 2, 100, 40), ColorPair::new(named_color(BLACK), named_color(BLACK)));
            batch.0.print(Point::new(45, 3), "= INVENTORY (ESC TO RETURN) =");

            let mut line = 1;
            for carry in (&carries).iter().filter(|c| c.owner == player_id) {
                let entry = (&is_item).get(carry.item).unwrap();
                batch.0.print(Point::new(41, line + 4), format!("{}) {}", line, entry.item.to_uppercase().as_str()));
                line += 1;
            }
        }
    }
}
