use std::collections::{HashSet, VecDeque};
use std::collections::hash_map::RandomState;
use bracket_terminal::prelude::INPUT;
use bracket_color::prelude::{BLACK, ColorPair, WHITE};
use bracket_lib::prelude::{Algorithm2D, BTerm, field_of_view_set, VirtualKeyCode, Point, Input, DrawBatch};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, Storage, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};
use bracket_color::prelude::RGB;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::ser::{Error, SerializeStruct};
use crate::colors::named_color;
use crate::commands::{FlowCommand, GameCommand, HackCommand};
use crate::entity::*;
use crate::game::{Batch, FlagExit, Store, WindowSize};
use crate::input::{InputSnapshot, InputSnapshots, KeyboardSnapshot, MouseSnapshot};
use crate::map::Map;
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::{CollectIntent, InvestigateIntent, IsDoor, IsItem, IsLocked};
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::glyph::Glyph;
use crate::{MAP_CONSOLE_LAYER, UI_CONSOLE_LAYER};
use crate::map_gen::run_map_gen;
use crate::rand_gen::{get_random_from, get_random_sub};
use crate::render_view::RenderView;
use crate::tiles::{MapTile, TileIndex};

pub struct IsCharacter;
pub struct HasSight(u8);

pub fn on_command_generate_level(mut storages: AllStoragesViewMut,) {
    use GameCommand::*;
    use FlowCommand::*;

    let front_is_generate_level = {
        storages.borrow::<UniqueView<VecDeque<GameCommand>>>().unwrap().front() == Some(&Flow(GenerateLevel))
    };

    if front_is_generate_level {
        let size = { *storages.borrow::<UniqueView<WindowSize>>().expect("WindowSize") };

        storages.clear();

        {
            let (new_map, storage) = run_map_gen(size.0.0, size.0.1 - 3, &mut storages.borrow::<UniqueViewMut<Store>>().expect("Store"));

            let all_doors = new_map.get_all_doors().as_slice().to_vec();
            let some_doors: HashSet<TileIndex, RandomState> = HashSet::from_iter(get_random_sub(new_map.get_all_doors().as_slice(), 0.5));

            for door in all_doors {
                let pos = new_map.index_to_point2d(door);
                let mut door_entity = storages.add_entity(
                    (IsDoor(true), HasPosition(pos), HasGlyph(Glyph::new('+'))));

                if some_doors.contains(&door) {
                    let pt = get_random_from(&storage.rects).center();
                    let key = format!("Key for {}", door);
                    let key_entity = storages.add_entity(
                        (IsItem { item: key, is_collected: false },
                         HasPosition(pt),
                         HasGlyph(Glyph::new('(')),
                        ));

                    storages.add_component(door_entity, (IsLocked { key: key_entity }, ));
                }
            }

            let starting_pos = get_random_from(&storage.rects).center();

            storages.add_entity(
                (IsPlayer,
                 IsCharacter,
                 HasPosition(starting_pos),
                 HasGlyph(Glyph::new('@')),
                 HasFieldOfView(Vec::new()), ));

            { *storages.borrow::<UniqueViewMut<Map>>().expect("Map") = new_map; }
        }

        { storages.borrow::<UniqueViewMut<IsDirty>>().expect("IsDirty").0 = true; }
        { storages.borrow::<UniqueViewMut<VecDeque<GameCommand>>>().expect("VecDeque<GameCommand>").pop_front(); }
    }
}

pub fn update_time(mut time: UniqueViewMut<Time>,) {
    time.0 += 1;
}

pub fn update_fields_of_view(map: UniqueView<Map>,
                             positions: View<HasPosition>,
                             sights: View<HasSight>,
                             mut fovs: ViewMut<HasFieldOfView>,) {

    for (id, (pos, mut fov)) in (&positions, &mut fovs).iter().with_id() {
        let sight = (&sights).get(id).unwrap_or(&HasSight(8));
        fov.0 = field_of_view_set(pos.0, sight.0 as i32, &*map).into_iter().collect()
    }
}

pub fn update_player_vision(mut map: UniqueViewMut<Map>,
                            is_player: View<IsPlayer>,
                            fovs: View<HasFieldOfView>, ) {

    for (_, fov) in (&is_player, &fovs).iter() {
        map.hide();
        map.show(&fov.0);
    }
}

pub fn interpret_player_input_as_bump_intent(keyboard: UniqueView<KeyboardSnapshot>,
                                             is_player: View<IsPlayer>,
                                             mut positions: ViewMut<HasPosition>,
                                             mut bump_intents: UniqueViewMut<VecDeque<BumpIntent>>,
                                             mut time: UniqueView<Time>, ) {

    for (id, (_, mut has_pos)) in (&is_player, &mut positions).iter().with_id() {
        let pos = has_pos.get_mut();

        let mut new_pos = Point::from(*pos);
        if keyboard.is_pressed(VirtualKeyCode::W) { new_pos.y -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::D) { new_pos.x += 1; }
        if keyboard.is_pressed(VirtualKeyCode::A) { new_pos.x -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::X)
            || keyboard.is_pressed(VirtualKeyCode::S) { new_pos.y += 1; }
        if keyboard.is_pressed(VirtualKeyCode::Q) { new_pos.x -= 1; new_pos.y -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::E) { new_pos.x += 1; new_pos.y -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::Z) { new_pos.x -= 1; new_pos.y += 1; }
        if keyboard.is_pressed(VirtualKeyCode::C) { new_pos.x += 1; new_pos.y += 1; }

        if *pos != new_pos {
            bump_intents.push_back(BumpIntent { id: time.0, bumper: id, pos: new_pos });
        }
    }
}

pub fn update_player_position(is_player: View<IsPlayer>,
                              positions: View<HasPosition>,
                              mut player_position: UniqueViewMut<PlayerPosition>,) {

    for (_, pos) in (&is_player, &positions).iter() {
        player_position.0 = pos.0;
    }
}

pub fn render_player_field_of_view(mut batch: UniqueViewMut<Batch>,
                                   mut map: UniqueViewMut<Map>,
                                   has_fov: View<HasFieldOfView>,
                                   is_player: View<IsPlayer>,) {

    if let Some((fov, _)) = (&has_fov, &is_player).iter().take(1).next() {
        batch.0.target(MAP_CONSOLE_LAYER);
        for pt in &fov.0 {
            if let Some(tile_index) = map.get_tile_index_from_point(*pt) {
                let glyph = match map.tiles[tile_index] {
                    MapTile::Obscured => '#',
                    MapTile::Corridor | MapTile::Floor(_) => '.',
                    MapTile::Door => '+',
                    _ => ' '
                };

                batch.0.set(*pt, ColorPair::new(RGB::named(WHITE), RGB::named(BLACK)), glyph as u16);
            }
        }
    }
}

pub fn render_player_visible_characters(mut batch: UniqueViewMut<Batch>,
                                        is_player: View<IsPlayer>,
                                        has_fov: View<HasFieldOfView>,
                                        character: View<IsCharacter>,
                                        positions: View<HasPosition>,
                                        glyphs: View<HasGlyph>,
                                        invisible: View<IsInvisible>,) {

    if let Some((fov, _)) = (&has_fov, &is_player).iter().take(1).next() {
        batch.0.target(MAP_CONSOLE_LAYER);
        for (_, has_pos, has_glyph, _) in (&character, &positions, &glyphs, !&invisible).iter() {
            let glyph = has_glyph.0;
            let pos = has_pos.0;
            if fov.0.contains(&pos) {
                batch.0.set(pos, ColorPair::new(glyph.fg, glyph.bg), glyph.ch as u16);
            }
        }
    }
}

pub fn clean_dirty(mut dirty: UniqueViewMut<IsDirty>) {
    dirty.0 = false;
}

pub fn submit_draw_batching(ctx: &mut BTerm,
                            mut batch: UniqueViewMut<Batch>) {
    use bracket_lib::prelude::render_draw_buffer;

    batch.0.submit(0).ok();
    render_draw_buffer(ctx).ok();
}
