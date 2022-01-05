use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::ops::Index;
use bracket_lib::prelude::{Point, Algorithm2D};
use shipyard::{AddEntity, AllStoragesViewMut, EntityId, Get, IntoIter, IntoWithId, Not, Remove, View, ViewMut};
use crate::entity::{HasPosition, IsInvisible};
use crate::game::Store;
use crate::game_systems::{BumpIntent, MoveDirective};
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

pub fn bump__collect_items((map, store): (&mut Map, &mut Store),
                           mut items: ViewMut<IsItem>,
                           mut has_position: ViewMut<HasPosition>,
                           mut has_inventory: ViewMut<HasInventory>,
                           mut is_invisible: ViewMut<IsInvisible>,
                           mut carries: ViewMut<CarriesItem>,) {

/*    let mut unhandled = Vec::new();
    let mut handled = false;

    while let Some(bump) = store.lpop::<BumpIntent>(BUMP_INTENT_REQUEST_QUEUE, 0) {
        let bumper = EntityId::from_inner(bump.entity).unwrap_or(EntityId::dead());
        if bumper == EntityId::dead() { continue; }

        for (id, mut inventory) in (&mut has_inventory).iter().with_id().filter(|(id, p)| *id == bumper) {
            assert_eq!(id, bumper);
            let mut ids = Vec::new();

            let bump_pos = Point::from(bump.pos);
            for (id, (mut item, _)) in (&mut items, &mut has_position).iter().with_id()
                .filter(|(_, (i, p))| !i.is_collected && p.0 == bump_pos) {

                inventory.0.insert(item.item.clone());
                println!("Collected {}", item.item);
                item.is_collected = true;
                ids.push(id);
            }

            if ids.len() > 0 {
                for id in ids {
                    has_position.remove(id);
                    is_invisible.add_entity(id, IsInvisible);
                    carries.add_entity(id, CarriesItem{ owner: bumper, item: id });
                }

                handled = true;
            }
        }

        if !handled {
            unhandled.push(bump);
        }
    }

    if unhandled.len() > 0 {
        store.lextend(BUMP_INTENT_REQUEST_QUEUE, &unhandled);
    }*/
}
