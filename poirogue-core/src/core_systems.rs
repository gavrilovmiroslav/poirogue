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
use crate::commands::{FlowCommand, GameCommand, GameplayContext, HackCommand};
use crate::entity::*;
use crate::game::{Batch, FlagAnimationDone, FlagExit, Store, TimeTracked, Timeline, WindowSize};
use crate::input::{InputSnapshot, InputSnapshots, KeyboardSnapshot, MouseSnapshot};
use crate::map::Map;
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::{CollectIntent, Intent, IsItem};
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::glyph::Glyph;
use crate::{MAP_CONSOLE_LAYER, UI_CONSOLE_LAYER};
use crate::game_systems::PlannedIntent::{Bump, Collect, Unlock};
use crate::map_gen::run_map_gen;
use crate::rand_gen::{get_random_between, get_random_from, get_random_sub};
use crate::render_view::RenderView;
use crate::tiles::{MapTile, TileIndex};

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

            fn obfuscate(s: &String) -> String {
                let mut ss = s.to_uppercase().clone();

                (0..s.len()).for_each(|index| {
                    if ss.chars().nth(index).unwrap() != ' ' {
                        if get_random_between(0, 100) < 10 {
                            ss.replace_range(index..index + 1, "?");
                        }
                    }
                });

                ss
            }

            for door in all_doors {
                let door_name = mnumonic::encode_u32_joined(door as u32);
                let obfuscated_door_name = obfuscate(&door_name);
                let obfuscated_key_name = obfuscate(&door_name);
                let pos = new_map.index_to_point2d(door);
                let mut door_entity = storages.add_entity((
                    IsDoor{ sign: obfuscated_door_name, is_closed: true, is_locked: None },
                    HasPosition(pos),
                    HasGlyph(Glyph::new('+')),
                ));


                if some_doors.contains(&door) {
                    let pt = get_random_from(&storage.rects).center();
                    let key = format!("key marked '{}'", obfuscated_key_name);
                    let key_entity = storages.add_entity(
                        (IsItem { item: key, is_collected: false },
                         HasPosition(pt),
                         HasGlyph(Glyph::new('(')),
                        ));

                    (&mut storages.borrow::<ViewMut<IsDoor>>().unwrap()).get(door_entity)
                        .map(|mut door| door.is_locked = Some(key_entity));
                }
            }

            let starting_pos = get_random_from(&storage.rects).center();

            storages.add_entity(
                (IsPlayer,
                 IsCharacter,
                 TimeTracked,
                 HasPosition(starting_pos),
                 HasGlyph(Glyph::new('@')),
                 HasSight{ sight_distance: 8, field_of_view: HashSet::new() }),
            );

            { *storages.borrow::<UniqueViewMut<Map>>().expect("Map") = new_map; }
        }

        { storages.borrow::<UniqueViewMut<IsDirty>>().expect("IsDirty").0 = true; }
        { storages.borrow::<UniqueViewMut<VecDeque<GameCommand>>>().expect("VecDeque<GameCommand>").pop_front(); }
    }
}

pub fn update_time(mut time: UniqueViewMut<Time>,
                   mut timeline: UniqueViewMut<Timeline>,
                   timed: View<TimeTracked>) {

    time.0 += 1;

    if timeline.is_empty() {
        for (id, _) in (&timed).iter().with_id() {
            timeline.add(id, 0);
        }
    }
}

pub fn interpret_player_input_as_bump_intent(keyboard: UniqueView<KeyboardSnapshot>,
                                             is_player: View<IsPlayer>,
                                             mut positions: ViewMut<HasPosition>,
                                             mut time: UniqueView<Time>,
                                             mut context: UniqueViewMut<GameplayContext>,
                                             mut bumps: UniqueViewMut<VecDeque<BumpIntent>>, ) {

    if *context != GameplayContext::MainGame { return; }

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
            bumps.push_back(BumpIntent { id: time.0, bumper: id, pos: new_pos });
        }
    }
}

pub fn interpret_player_input_as_pickup(keyboard: UniqueView<KeyboardSnapshot>,
                                        is_player: View<IsPlayer>,
                                        items: View<IsItem>,
                                        mut positions: ViewMut<HasPosition>,
                                        mut collects: UniqueViewMut<VecDeque<CollectIntent>>,
                                        mut time: UniqueView<Time>, ) {

    if keyboard.is_pressed(VirtualKeyCode::Comma) {
        let (player_id, (_, player_pos)) = (&is_player, &positions).iter().with_id().take(1).next().unwrap();

        for (item_id, _) in (&items, &positions).iter().with_id().filter(|(_, (item, pos))| pos.0 == player_pos.0) {
            collects.push_back(CollectIntent { id: time.0, item: item_id, collector: player_id });
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

pub fn update_fields_of_view(mut positions: ViewMut<HasPosition>,
                             mut sights: ViewMut<HasSight>,
                             mut dirty: UniqueViewMut<IsDirty>,
                             map: UniqueView<Map>) {

    if dirty.0 {
        for (pos, mut sight) in (&positions, &mut sights).iter() {
            sight.field_of_view = field_of_view_set(pos.0, sight.sight_distance as i32, &*map);
        }
    }
}

pub fn render_player_field_of_view(mut batch: UniqueViewMut<Batch>,
                                   mut map: UniqueViewMut<Map>,
                                   has_sight: View<HasSight>,
                                   is_player: View<IsPlayer>,) {

    if let Some((sight, _)) = (&has_sight, &is_player).iter().take(1).next() {
        batch.0.target(MAP_CONSOLE_LAYER);
        for pt in &sight.field_of_view {
            if let Some(tile_index) = map.get_tile_index_from_point(*pt) {
                let glyph = match map.tiles[tile_index] {
                    MapTile::Obscured => '#',
                    MapTile::Corridor | MapTile::Floor(_) => '.',
                    MapTile::Door => '+',
                    _ => ' '
                };

                batch.0.set(*pt, ColorPair::new(RGB::from((140, 90, 90)), RGB::named(BLACK)), glyph as u16);
            }
        }
    }
}

pub fn render_player_visible_characters(mut batch: UniqueViewMut<Batch>,
                                        is_player: View<IsPlayer>,
                                        has_sight: View<HasSight>,
                                        character: View<IsCharacter>,
                                        positions: View<HasPosition>,
                                        glyphs: View<HasGlyph>,) {

    if let Some((sight, _)) = (&has_sight, &is_player).iter().take(1).next() {
        batch.0.target(MAP_CONSOLE_LAYER);
        for (_, has_pos, has_glyph,) in (&character, &positions, &glyphs,).iter() {
            let glyph = has_glyph.0;
            let pos = has_pos.0;
            if sight.field_of_view.contains(&pos) {
                batch.0.set(pos, ColorPair::new(glyph.fg, glyph.bg), glyph.ch as u16);
            }
        }
    }
}

pub fn clean_dirty(mut dirty: UniqueViewMut<IsDirty>, mut anim: UniqueViewMut<FlagAnimationDone>) {
    dirty.0 = false;
    anim.0 = true;
}

pub fn submit_draw_batching(ctx: &mut BTerm,
                            mut batch: UniqueViewMut<Batch>) {
    use bracket_lib::prelude::render_draw_buffer;

    batch.0.submit(0).ok();
    render_draw_buffer(ctx).ok();
}
