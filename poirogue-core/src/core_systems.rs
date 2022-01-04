use std::collections::VecDeque;
use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::{Algorithm2D, BTerm, field_of_view_set, VirtualKeyCode, Point, Input};
use shipyard::{AddEntity, AllStoragesViewMut, EntityId, IntoIter, IntoWithId, View, ViewMut, World};
use bracket_color::prelude::RGB;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::ser::{Error, SerializeStruct};
use crate::colors::named_color;
use crate::commands::{FlowCommand, GameCommand, HackCommand};
use crate::entity::*;
use crate::game::{GameData, Store};
use crate::input::{InputSnapshot, InputSnapshots};
use crate::map::Map;
use crate::POSITION_QUERY_REQUEST_QUEUE;
use crate::render_view::RenderView;
use crate::tiles::{DoorState, MapTile, TileIndex};

pub fn accept_meta_commands((input, comms): (&InputSnapshots, &mut VecDeque<GameCommand>)) {
    if input.keyboard.is_released(VirtualKeyCode::Escape) {
        comms.push_back(GameCommand::Flow(FlowCommand::Exit));
    }

    if input.keyboard.is_pressed(VirtualKeyCode::Return) {
        comms.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));
    }

    if input.keyboard.is_pressed(VirtualKeyCode::Tab) {
        comms.push_back(GameCommand::Flow(FlowCommand::CycleViews));
    }

    if input.keyboard.is_pressed(VirtualKeyCode::F2) {
        comms.push_back(GameCommand::Hack(HackCommand::UnlockAllDoors));
    }

    if input.keyboard.is_pressed(VirtualKeyCode::F3) {
        comms.push_back(GameCommand::Hack(HackCommand::LockAllDoors));
    }

    if input.keyboard.is_pressed(VirtualKeyCode::F4) {
        comms.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));
    }

    if input.keyboard.is_pressed(VirtualKeyCode::F5) {
        comms.push_back(GameCommand::Flow(FlowCommand::ReloadViewConfigs));
    }
}


pub fn update_dirty_fovs((store, map): (&Store, &Map),
                         positions: View<HasPosition>, mut fovs: ViewMut<HasFieldOfView>, dirty: View<IsDirty>) {

    for (pos, mut fov, _) in (&positions, &mut fovs, &dirty).iter().filter(|(_, _, d)| d.is_dirty()) {
        fov.0 = field_of_view_set(pos.0, store.get("fov").unwrap_or(16), map).into_iter().collect()
    }
}


pub fn update_revealed_map(map: &mut Map, _: View<IsPlayer>, fovs: View<HasFieldOfView>, dirty: View<IsDirty>) {
    for (fov, _) in (&fovs, &dirty).iter().filter(|(_, d)| d.is_dirty()) {
        map.hide();
        map.show(&fov.0);
    }
}


#[derive(Serialize, Deserialize)]
pub struct QueryPositionRequest {
    pub entity: u64,
    pub pos: (i32, i32),
}

#[derive(Serialize, Deserialize)]
pub enum Motion {
    Unrest,
    MoveTo(TileIndex),
}


pub fn handle_player_controls((input, store): (&InputSnapshots, &mut Store),
                              _: View<IsPlayer>, mut positions: ViewMut<HasPosition>) {

    for (id, mut has_pos) in (&mut positions).iter().with_id() {
        let keyboard = &input.keyboard;
        let pos = has_pos.get_mut();

        let mut new_pos = Point::from(*pos);
        if keyboard.is_pressed(VirtualKeyCode::Up) { new_pos.y -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::Down) { new_pos.y += 1; }
        if keyboard.is_pressed(VirtualKeyCode::Left) { new_pos.x -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::Right) { new_pos.x += 1; }

        if *pos != new_pos {
            store.ladd(POSITION_QUERY_REQUEST_QUEUE, &QueryPositionRequest { entity: id.inner(), pos: new_pos.to_tuple() });
        }
    }
}


pub fn query_position_for_wall_blocking((map, store): (&Map, &mut Store), mut storage: AllStoragesViewMut) {
    let mut returns = Vec::new();

    while let Some(item) = store.lpop::<QueryPositionRequest>(POSITION_QUERY_REQUEST_QUEUE, 0) {
        let index = map.point2d_to_index(Point::from(item.pos));

        if let Some(entity) = EntityId::from_inner(item.entity) {
            if !map.is_tile_blocked(index) {
                storage.add_component(entity, (Motion::MoveTo(index), ));
            } else {
                returns.push(item);
            }
        }
    }

    if returns.len() > 0 {
        store.lextend(POSITION_QUERY_REQUEST_QUEUE, &returns);
    }
}


pub fn query_position_for_door_opening((map, store): (&mut Map, &mut Store), mut storage: AllStoragesViewMut) {
    let mut returns = Vec::new();

    while let Some(item) = store.lpop::<QueryPositionRequest>(POSITION_QUERY_REQUEST_QUEUE, 0) {
        let index = map.point2d_to_index(Point::from(item.pos));

        if let Some(entity) = EntityId::from_inner(item.entity) {
            if let MapTile::Door(DoorState::Closed) = map.tiles[index] {
                map.tiles[index] = MapTile::Door(DoorState::Open);
                storage.add_component(entity, (Motion::Unrest,));
            } else {
                returns.push(item);
            }
        }
    }

    if returns.len() > 0 {
        store.lextend(POSITION_QUERY_REQUEST_QUEUE, &returns);
    }
}


pub fn handle_move_to_commands(map: &Map, mut positions: ViewMut<HasPosition>, mut dirty: ViewMut<IsDirty>, mut motions: ViewMut<Motion>) {
    for (mut pos, mut dirt, mov) in (&mut positions, &mut dirty, &motions).iter() {
        if let Motion::MoveTo(index) = mov {
            let pt = map.index_to_point2d(*index);
            pos.0 = pt;
        }

        dirt.mark();
    }

    motions.clear();
}


pub fn render_map((map, store, ctx): (&mut Map, &Store, &mut BTerm)) {
    fn render_map_layer(map: &mut Map, store: &Store, ctx: &mut BTerm) {
        let view = store.get::<RenderView>("view")
            .unwrap_or(RenderView::Game);

        ctx.set_active_console(0);
        ctx.cls();
        map.render(ctx, &view, &store);
    }

    fn render_fps_count(ctx: &mut BTerm) {
        ctx.set_active_console(2);
        ctx.cls();
        ctx.print_color(1, 1, named_color(WHITE), named_color(BLACK), format!("FPS: {}", ctx.fps));
    }

    render_map_layer(map, store, ctx);
    render_fps_count(ctx);
}


pub fn render_entities((map, ctx): (&Map, &mut BTerm), positions: View<HasPosition>, glyphs: View<HasGlyph>) {
    for (has_pos, has_glyph) in (&positions, &glyphs).iter() {
        let glyph = has_glyph.0;
        let pos = has_pos.0;
        let index = map.point2d_to_index(pos);
        if map.is_tile_revealed(index) && map.is_tile_visible(index) {
            ctx.print_color(pos.x, pos.y, RGB::from(glyph.fg), RGB::from(glyph.bg), glyph.ch);
        }
    }
}


pub fn clean_dirty(mut dirty: ViewMut<IsDirty>) {
    for (mut dirt) in (&mut dirty).iter().filter(|(d)| d.is_dirty()) {
        dirt.clean();
    }
}
