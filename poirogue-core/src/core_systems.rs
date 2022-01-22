use std::collections::{HashSet, VecDeque};
use std::collections::hash_map::RandomState;
use bracket_terminal::prelude::INPUT;
use bracket_color::prelude::{BLACK, ColorPair, WHITE};
use bracket_lib::prelude::{Algorithm2D, BTerm, field_of_view_set, VirtualKeyCode, Point, Input, DrawBatch};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, IntoWithId, NonSendSync, Storage, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};
use bracket_color::prelude::RGB;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::ser::{Error, SerializeStruct};
use crate::colors::{ColorShifter, named_color};
use crate::commands::{FlowCommand, GameCommand, HackCommand};
use crate::entity::*;
use crate::game::{Batch, BinaryData, FlagExit, FlagRecompileScripts, Scripting, Store, WindowSize};
use crate::input::{InputSnapshot, InputSnapshots, KeyboardSnapshot, MouseSnapshot};
use crate::map::Map;
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::{CollectIntent, InvestigateIntent, IsDoor, IsItem, IsLocked, NotificationLog};
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::glyph::Glyph;
use crate::{MAP_CONSOLE_LAYER, UI_CONSOLE_LAYER};
use crate::map_gen::run_map_gen;
use crate::rand_gen::{get_random_from, get_random_sub};
use crate::tiles::{MapTile, TileIndex};
use rhai::{Array, AST, Dynamic, Engine, Scope};

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
                 HasFieldOfView(Vec::new()),
                 HasSight(16),
                 Vision("normal_vision.script".to_string()), ));

            { *storages.borrow::<UniqueViewMut<Map>>().expect("Map") = new_map; }
        }

        { storages.borrow::<UniqueViewMut<IsDirty>>().expect("IsDirty").0 = true; }
        { storages.borrow::<UniqueViewMut<VecDeque<GameCommand>>>().expect("VecDeque<GameCommand>").pop_front(); }
    }
}

pub fn update_time(mut time: UniqueViewMut<Time>,) {
    time.0 += 1;
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

pub struct Vision(pub String);

fn vec_u8_to_str(us: Vec<u8>) -> String {
    String::from_utf8(us).unwrap()
}

use lazy_static::*;
use std::sync::Mutex;
use log::Log;
use lru::{DefaultHasher, LruCache};

lazy_static! {
    static ref AST_LRU: Mutex<LruCache<&'static str, AST>> = Mutex::new(LruCache::with_hasher(2, DefaultHasher::default()));
}

pub fn clear_ast_lru_cache_if_requested(mut recompile: UniqueViewMut<FlagRecompileScripts>,
                                        mut is_dirty: UniqueViewMut<IsDirty>) {
    if recompile.0 {
        let mut cache = AST_LRU.lock().unwrap();
        cache.clear();
        recompile.0 = false;
        is_dirty.0 = true;
    }
}

pub fn get_from_cache_or_recompile_script(cache_name: &'static str, script_name: &String, data: &BinaryData, engine: &Engine) -> Result<AST, rhai::ParseError> {

    let mut cache = AST_LRU.lock().unwrap();
    if !cache.contains(&cache_name) {
        let perception_code = vec_u8_to_str(data.0.get(script_name.as_str()).expect(format!("Script unknown: {}", script_name).as_str()));
        match engine.compile(perception_code.as_str()) {
            Ok(ast) => {
                cache.put(cache_name, ast.clone());
                Ok(ast)
            },

            Err(err) => {
                Err(err)
            }
        }
    } else {
        Ok(cache.get(&cache_name).unwrap().clone())
    }
}

pub fn render_player_field_of_view(mut batch: UniqueViewMut<Batch>,
                                   mut scripting: UniqueViewMut<Scripting>,
                                   mut map: UniqueViewMut<Map>,
                                   mut has_fov: ViewMut<HasFieldOfView>,
                                   sights: View<HasSight>,
                                   data: UniqueView<BinaryData>,
                                   is_player: View<IsPlayer>,
                                   has_pos: View<HasPosition>,
                                   visions: View<Vision>,) {

    if let Some((vision, pos, sight, mut fov, _)) = (&visions, &has_pos, &sights, &mut has_fov, &is_player).iter().take(1).next() {
        let ast = get_from_cache_or_recompile_script(
            "render_player_field_of_view_ast", &vision.0, &data, &scripting.0);

        match ast {
            Ok(ast) => {
                let mut scope = Scope::new();
                batch.0.target(MAP_CONSOLE_LAYER);

                fov.0 = field_of_view_set(pos.0, sight.0 as i32, &*map).into_iter().collect();

                let fov_list: Array = {
                    fov.0.iter().map(|pt| {
                        let tile_name = map.get_tile_at_point(&pt).name();

                        Dynamic::from_iter(vec![
                            Dynamic::from(pt.clone()), Dynamic::from(tile_name),
                        ])
                    }).collect()
                };

                let result: Result<Array, _> = scripting.0.call_fn(&mut scope, &ast, "perceive", (fov_list, sight.0, pos.0));

                if let Ok(pts) = result {
                    for pt in pts {
                        let arr = pt.clone_cast::<Array>();
                        let pt = arr[0].clone_cast::<Point>();
                        let ch = arr[1].clone_cast::<char>() as u16;
                        let fg = arr[2].clone_cast::<RGB>();
                        let bg = arr[3].clone_cast::<RGB>();
                        batch.0.set(pt, ColorPair::new(fg, bg), ch);
                    }
                } else {
                    println!("{:?}", result.err().unwrap());
                }
            },

            Err(err) => {
                println!("{:?}", err);
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
