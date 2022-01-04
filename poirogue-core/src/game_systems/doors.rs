use bracket_color::prelude::BLACK;
use bracket_lib::prelude::{Point, Algorithm2D, BTerm, RED, DARK_RED};
use shipyard::{AllStoragesViewMut, EntityId, Get, View, ViewMut};
use crate::{BUMP_INTENT_REQUEST_QUEUE, UNLOCK_INTENT_REQUEST_QUEUE};
use crate::colors::named_color;
use crate::entity::IsDirty;
use crate::game::Store;
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::HasInventory;
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::map::Map;
use crate::store_helpers::StoreHelpers;
use crate::tiles::{DoorState, MapTile, TileIndex};

pub fn get_required_lock_string(index: TileIndex) -> String {
    format!("door@{}:requires_lock", index)
}

pub fn render_locked_doors((map, store, ctx): (&mut Map, &mut Store, &mut BTerm)) {
    for door in map.get_all_closed_doors() {
         if store.exists(get_required_lock_string(door).as_str()) {
             let pt = map.index_to_point2d(door);
             if map.is_tile_revealed(door) {
                 if map.is_tile_visible(door) {
                     ctx.print_color(pt.x, pt.y, named_color(RED), named_color(BLACK), '+');
                 } else {
                     ctx.print_color(pt.x, pt.y, named_color(DARK_RED), named_color(BLACK), '+');
                 }
             }
         }
    }
}


pub fn bump__door_unlock_intent((map, store): (&mut Map, &mut Store), mut storage: AllStoragesViewMut) {
    let mut unhandled = Vec::new();

    while let Some(item) = store.lpop::<BumpIntent>(BUMP_INTENT_REQUEST_QUEUE, 0) {
        let index = map.point2d_to_index(Point::from(item.pos));

        if let Some(entity) = EntityId::from_inner(item.entity) {
            if let MapTile::Door(DoorState::Closed) = map.tiles[index] {
                if store.exists(get_required_lock_string(index).as_str()) {
                    store.ladd(UNLOCK_INTENT_REQUEST_QUEUE, &UnlockIntent { entity: item.entity, tile: index });
                } else {
                    unhandled.push(item);
                }
            } else {
                unhandled.push(item);
            }
        }
    }

    if unhandled.len() > 0 {
        store.lextend(BUMP_INTENT_REQUEST_QUEUE, &unhandled);
    }
}


pub fn bump__open_unlocked_doors((map, store): (&mut Map, &mut Store), mut storage: AllStoragesViewMut) {
    let mut unhandled = Vec::new();

    while let Some(item) = store.lpop::<BumpIntent>(BUMP_INTENT_REQUEST_QUEUE, 0) {
        let index = map.point2d_to_index(Point::from(item.pos));

        if let Some(entity) = EntityId::from_inner(item.entity) {
            if let MapTile::Door(DoorState::Closed) = map.tiles[index] {
                map.tiles[index] = MapTile::Door(DoorState::Open);
                storage.add_component(entity, (MoveDirective::MoveBy(0, 0), ));
            } else {
                unhandled.push(item);
            }
        }
    }

    if unhandled.len() > 0 {
        store.lextend(BUMP_INTENT_REQUEST_QUEUE, &unhandled);
    }
}


pub fn unlock__if_has_key_for_door((map, store): (&mut Map, &mut Store), mut storage: AllStoragesViewMut) {
    let mut unhandled = Vec::new();

    while let Some(item) = store.lpop::<UnlockIntent>(UNLOCK_INTENT_REQUEST_QUEUE, 0) {
        let index = item.tile;
        let mut handled = false;

        if let Some(entity) = EntityId::from_inner(item.entity) {
            let required_lock = get_required_lock_string(index);
            let required_key = store.get(required_lock.as_str()).unwrap_or("".to_string());
            let (mut has_inventory, mut is_dirty) = storage.borrow::<(ViewMut<HasInventory>, ViewMut<IsDirty>)>().unwrap();

            if let Ok((mut inventory, mut dirt)) = (&mut has_inventory, &mut is_dirty).get(entity) {
                if let MapTile::Door(DoorState::Closed) = map.tiles[index] {
                    if inventory.0.contains(&required_key) {
                        println!("You use {} to unlock the door. The key crumbles to dust.", required_key);
                        inventory.0.remove(&required_key);
                        store.rem(required_lock.as_str());
                        map.tiles[index] = MapTile::Door(DoorState::Open);
                        dirt.mark();
                        handled = true;
                    } else {
                        println!("The door is locked. {} is required to unlock it.", required_key);
                    }
                }
            }
        }

        if !handled {
            unhandled.push(item);
        }
    }

    if unhandled.len() > 0 {
        store.lextend(UNLOCK_INTENT_REQUEST_QUEUE, &unhandled);
    }
}


pub fn unlock__default(store: &mut Store) {
    store.lregen(UNLOCK_INTENT_REQUEST_QUEUE);
}