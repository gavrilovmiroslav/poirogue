use std::collections::{HashSet, VecDeque};
use std::collections::hash_map::RandomState;
use bracket_terminal::prelude::INPUT;
use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::{Algorithm2D, BTerm, field_of_view_set, VirtualKeyCode, Point, Input};
use shipyard::{AddEntity, AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, IntoWithId, Storage, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};
use bracket_color::prelude::RGB;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::ser::{Error, SerializeStruct};
use crate::colors::named_color;
use crate::commands::{FlowCommand, GameCommand, HackCommand};
use crate::entity::*;
use crate::game::{FlagExit, Store, WindowSize};
use crate::input::{InputSnapshot, InputSnapshots, KeyboardSnapshot, MouseSnapshot};
use crate::map::Map;
use crate::game_systems::directives::MoveDirective;
use crate::game_systems::{CollectIntent, Handle, InvestigateIntent, IsDoor, IsItem, IsLocked};
use crate::game_systems::intents::{BumpIntent, UnlockIntent};
use crate::glyph::Glyph;
use crate::map_gen::run_map_gen;
use crate::rand_gen::{get_random_from, get_random_sub};
use crate::render_view::RenderView;
use crate::tiles::{MapTile, TileIndex};

pub struct IsCharacter;

pub fn on_input_keyboard_generate_level(mut storages: AllStoragesViewMut,) {

    let size = { *storages.borrow::<UniqueView<WindowSize>>().expect("WindowSize") };

    if storages.borrow::<UniqueView<KeyboardSnapshot>>().expect("KeyboardSnapshot").is_pressed(VirtualKeyCode::F4) {
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
    }
}

pub fn make_input_snapshots(mut keyboard: UniqueViewMut<KeyboardSnapshot>,
                            mut mouse: UniqueViewMut<MouseSnapshot>) {
    use std::borrow::Borrow;
    keyboard.update(INPUT.lock().borrow());
    mouse.update(INPUT.lock().borrow());
}

pub fn on_input_keyboard_exit(keyboard: UniqueView<KeyboardSnapshot>,
                              mut exit: UniqueViewMut<FlagExit>,) {

    if keyboard.is_released(VirtualKeyCode::Escape) {
        exit.0 = true;
    }
}

pub fn update_dirty_fovs(store: UniqueView<Store>,
                         map: UniqueView<Map>,
                         positions: View<HasPosition>,
                         mut fovs: ViewMut<HasFieldOfView>,
                         dirty: UniqueView<IsDirty>) {

    if dirty.0 {
        let m: &Map = &map;
        for (pos, mut fov) in (&positions, &mut fovs).iter() {
            fov.0 = field_of_view_set(pos.0, store.0.get("fov").unwrap_or(16), m).into_iter().collect()
        }
    }
}


pub fn update_player_vision(mut map: UniqueViewMut<Map>,
                            is_player: View<IsPlayer>,
                            fovs: View<HasFieldOfView>,
                            dirty: UniqueView<IsDirty>) {
    if dirty.0 {
        for (_, fov) in (&is_player, &fovs).iter() {
            map.hide();
            map.show(&fov.0);
        }
    }
}


pub fn interpret_player_input_as_bump_intent(keyboard: UniqueView<KeyboardSnapshot>,
                                             is_player: View<IsPlayer>,
                                             mut positions: ViewMut<HasPosition>,
                                             mut bump_intents: ViewMut<Handle<BumpIntent>>,
                                             mut entities: EntitiesViewMut,) {

    for (id, (_, mut has_pos)) in (&is_player, &mut positions).iter().with_id() {
        let pos = has_pos.get_mut();

        let mut new_pos = Point::from(*pos);
        if keyboard.is_pressed(VirtualKeyCode::Up) { new_pos.y -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::Down) { new_pos.y += 1; }
        if keyboard.is_pressed(VirtualKeyCode::Left) { new_pos.x -= 1; }
        if keyboard.is_pressed(VirtualKeyCode::Right) { new_pos.x += 1; }

        if *pos != new_pos {
            entities.add_entity((&mut bump_intents, ), (Handle::new(BumpIntent { bumper: id, pos: new_pos }),));
        }
    }
}

pub fn update_time(mut time: UniqueViewMut<Time>,) {
    time.0 += 1;
}

pub fn update_player_position(is_player: View<IsPlayer>,
                              positions: View<HasPosition>,
                              dirty: UniqueView<IsDirty>,
                              mut player_position: UniqueViewMut<PlayerPosition>,) {

    if dirty.0 {
        for (_, pos) in (&is_player, &positions).iter() {
            player_position.0 = pos.0;
        }
    }
}


pub fn render_map(ctx: &mut BTerm,
                  map: UniqueView<Map>,
                  store: UniqueView<Store>,
                  dirty: UniqueView<IsDirty>,
                  player_position: UniqueView<PlayerPosition>,
                  time: UniqueView<Time>) {

    if dirty.0 {
        let view = store.0.get::<RenderView>("view")
            .unwrap_or(RenderView::Game);

        ctx.cls();
        map.render(ctx, &view, &store, player_position.0, time.0);
    }
}


pub fn render_characters(ctx: &mut BTerm,
                         map: UniqueView<Map>,
                         character: View<IsCharacter>,
                         positions: View<HasPosition>,
                         glyphs: View<HasGlyph>,
                         invisible: View<IsInvisible>,
                         dirty: UniqueView<IsDirty>,) {

    if dirty.0 {
        for (_, has_pos, has_glyph, _) in (&character, &positions, &glyphs, !&invisible).iter() {
            let glyph = has_glyph.0;
            let pos = has_pos.0;
            let index = map.point2d_to_index(pos);
            if map.is_tile_revealed(index) && map.is_tile_visible(index) {
                ctx.print_color(pos.x, pos.y, RGB::from(glyph.fg), RGB::from(glyph.bg), glyph.ch);
            }
        }
    }
}


pub fn clean_dirty(mut dirty: UniqueViewMut<IsDirty>) {
    dirty.0 = false;
}

