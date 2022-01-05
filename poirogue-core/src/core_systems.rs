use std::borrow::BorrowMut;
use std::collections::VecDeque;
use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::{Algorithm2D, BTerm, field_of_view_set, VirtualKeyCode, Point, Input};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, IntoWithId, View, ViewMut, World};
use bracket_color::prelude::RGB;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::ser::{Error, SerializeStruct};
use crate::colors::named_color;
use crate::commands::{FlowCommand, GameCommand, HackCommand};
use crate::entity::*;
use crate::game::{GameData, Store};
use crate::input::{InputSnapshot, InputSnapshots};
use crate::map::Map;
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::render_view::RenderView;
use crate::tiles::{MapTile, TileIndex};

pub struct IsCharacter;

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
                         positions: View<HasPosition>,
                         mut fovs: ViewMut<HasFieldOfView>,
                         dirty: View<IsDirty>) {

    for (pos, mut fov, _) in (&positions, &mut fovs, &dirty).iter() {
        fov.0 = field_of_view_set(pos.0, store.get("fov").unwrap_or(16), map).into_iter().collect()
    }
}


pub fn update_player_vision(map: &mut Map,
                            is_player: View<IsPlayer>,
                            fovs: View<HasFieldOfView>,
                            dirty: View<IsDirty>) {
    for (_, fov, _) in (&is_player, &fovs, &dirty).iter() {
        map.hide();
        map.show(&fov.0);
    }
}


pub fn interpret_player_bump_controls(input: &InputSnapshots,
                                      is_player: View<IsPlayer>,
                                      mut positions: ViewMut<HasPosition>,
                                      mut bump_intents: ViewMut<BumpIntent>,
                                      mut entities: EntitiesViewMut,) {

    for (id, (_, mut has_pos)) in (&is_player, &mut positions).iter().with_id() {
        let keyboard = &input.keyboard;
        let pos = has_pos.get_mut();

        let mut new_pos = Point::from(*pos);
        if keyboard.is_pressed(VirtualKeyCode::Up) { new_pos.y -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::Down) { new_pos.y += 1; }
        if keyboard.is_pressed(VirtualKeyCode::Left) { new_pos.x -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::Right) { new_pos.x += 1; }

        if *pos != new_pos {
            entities.add_entity((&mut bump_intents, ), (BumpIntent { bumper: id, pos: new_pos }, ));
        }
    }
}


pub fn update_stored_player_position(store: &mut Store,
                                     is_player: View<IsPlayer>,
                                     positions: View<HasPosition>,
                                     dirty: View<IsDirty>,) {

    for (_, pos, _) in (&is_player, &positions, &dirty).iter() {
        store.set("player_position", &(pos.0.x, pos.0.y)).unwrap();
    }
}


pub fn render_map((map, store, ctx): (&mut Map, &mut Store, &mut BTerm),
                  is_player: View<IsPlayer>,
                  dirty: View<IsDirty>,) {

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

    let is_player_dirty = (&is_player, &dirty).iter().collect::<Vec<_>>().len() > 0;

    if is_player_dirty || store.get("debug_render_dirty").unwrap_or(false) {
        render_map_layer(map, store, ctx);
        store.rem("debug_render_dirty").unwrap();
    }

    render_fps_count(ctx);
}


pub fn render_characters((map, ctx): (&Map, &mut BTerm),
                         character: View<IsCharacter>,
                         positions: View<HasPosition>,
                         glyphs: View<HasGlyph>,
                         invisible: View<IsInvisible>,) {

    for (_, has_pos, has_glyph, _) in (&character, &positions, &glyphs, !&invisible).iter() {
        let glyph = has_glyph.0;
        let pos = has_pos.0;
        let index = map.point2d_to_index(pos);
        if map.is_tile_revealed(index) && map.is_tile_visible(index) {
            ctx.print_color(pos.x, pos.y, RGB::from(glyph.fg), RGB::from(glyph.bg), glyph.ch);
        }
    }
}


pub fn clean_dirty(mut dirty: ViewMut<IsDirty>) {
    dirty.clear();
}
