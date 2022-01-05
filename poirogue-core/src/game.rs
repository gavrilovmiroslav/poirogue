use std::str;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::collections::hash_map::RandomState;
use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use bracket_terminal::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use bracket_lib::prelude::*;
use caves::{Cave, FileCave, MemoryCave};
use object_pool::{Pool, Reusable};
use lazy_static::*;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{de, Serialize};
use shipyard::{Workload, WorkloadBuilder, World};
use simple_ringbuf::RingBuffer;

use crate::map::Map;
use crate::readonly_archive_cave::ReadonlyArchiveCave;
use crate::commands::{ActionCommand, FlowCommand, GameCommand, GameFlow, HackCommand};
use crate::input::{InputSnapshotState, InputSnapshot, KeyboardSnapshot, InputSnapshots};
use crate::map_gen::run_map_gen;
use crate::murder_gen::generate_murder;
use crate::{core_systems, rand_gen};
use crate::colors::named_color;
use crate::core_systems::IsCharacter;
use crate::entity::{HasFieldOfView, HasGlyph, HasPosition, IsDirty, IsPlayer};
use crate::glyph::Glyph;
use crate::json::JsonFields;
//use crate::entity::{AbstractEntity, Entity};
use crate::opt::Opt;
use crate::rand_gen::{get_random_between, get_random_from, get_random_sub};
use crate::rex::draw_rex;
use crate::tiles::{MapTile, MapTileRep, TileIndex};
use crate::render_view::{View};
use crate::render_view::*;
use crate::store_helpers::StoreHelpers;
use crate::game_systems;
use crate::game_systems::{HasInventory, IsDoor, IsItem, IsLocked, Item, ItemSpendMode, NotificationLog};

pub type Store = PickleDb;

pub struct Game {
    pub data: GameData,
}

pub struct GameData {
    pub dirty: bool,
    pub size: (i32, i32),
    pub map: Map,
    pub flow: GameFlow,
    pub commands: VecDeque<GameCommand>,
    pub input: InputSnapshots,
    pub data: Box<dyn Cave>,
    pub store: Store,
    pub world: World,
}

impl JsonFields for GameData {
    fn get_json<T>(&self, name: &str) -> Option<T>
        where for <'a> T: de::Deserialize<'a> {
        if let Ok(binary_data) = self.data.get(name) {
            if let Ok(string_data) = str::from_utf8(&binary_data) {
                let json_as_struct = serde_json::from_str::<T>(string_data).unwrap();
                return Some(json_as_struct)
            }
        }

        return None;
    }

    fn set_json<T>(&self, name: &str, value: &T)
        where T: ?Sized + Serialize {

        if let Ok(json) = serde_json::to_string_pretty(value) {
            self.data.set(name, &json.as_bytes().to_vec()).unwrap();
        }
    }
}

impl GameData {
    pub fn new(w: i32, h: i32, args: &Opt) -> GameData {
        GameData {
            dirty: true,
            size: (w, h),
            map: Map::new(w, h),
            flow: GameFlow::Player,
            commands: VecDeque::default(),
            input: InputSnapshots::default(),
            data: if args.release_mode {
                Box::new(ReadonlyArchiveCave::open(format!("{}.bin", args.data_directory)))
            } else {
                Box::new(FileCave::new(Path::new(args.data_directory.as_str())).unwrap())
            },
            store: PickleDb::new("", PickleDbDumpPolicy::NeverDump, SerializationMethod::Bin),
            world: World::new(),
        }
    }

    pub fn setup_store(&mut self) {
        { /* FOV */
            let fov = 16;
            self.store.set("fov", &fov)
                .expect("Failed storing FOV");
        }

        { /* NOISE MAP */
            let size = (self.map.width * self.map.height) as usize;
            let mut noise = Vec::with_capacity(size);
            for _ in 0..size {
                noise.push(get_random_between(0.0, 1.0));
            }

            self.store.set("noise_map", &noise)
                .expect("Failed storing noise map");
        }

        { /* TIME */
            self.store.set("time", &0.0);
        }
    }

    pub fn register_render_view(&self, view: RenderView) {
        fn view_to_file(view: &RenderView) -> &str {
            match view {
                RenderView::Game => "game.view.json",
                RenderView::Debug => "debug.view.json",
            }
        }

        let filename = view_to_file(&view);
        let rep = if let Some(json) = self.get_json::<MapTileRep>(filename) {
            json
        } else {
            self.set_json(filename, &MapTileRep::default());
            MapTileRep::default()
        };

        cache_render_view_rep(view, rep);
    }

    fn create_new_level(&mut self) {
        self.world.clear();

        let (map, storage) = run_map_gen(self.size.0, self.size.1 - 3, &mut self.store);
        self.map = map;

        let all_doors = self.map.get_all_doors().as_slice().to_vec();
        let some_doors: HashSet<TileIndex, RandomState> = HashSet::from_iter(get_random_sub(self.map.get_all_doors().as_slice(), 0.5));

        for door in all_doors {
            let pos = self.map.index_to_point2d(door);
            let mut door_entity = self.world.add_entity((
                IsDoor(true),
                HasPosition(pos),
                HasGlyph(Glyph::new('+')),
            ));

            if some_doors.contains(&door) {
                let pt = get_random_from(&storage.rects).center();
                let key = format!("Key for {}", door);
                let key_entity = self.world.add_entity((
                    IsItem{ item: key.clone(), is_collected: false },
                    HasPosition(pt.clone()),
                    HasGlyph(Glyph::new('(')),
                ));

                self.world.add_component(door_entity, (IsLocked(key_entity, ItemSpendMode::Consume),));
                println!("Added {} to the world at ({}, {})", key, pt.x, pt.y);
            }
        }

        let starting_pos = get_random_from(&storage.rects).center();

        self.world.add_entity((
            IsPlayer,
            IsCharacter,
            IsDirty,
            HasPosition(starting_pos),
            HasGlyph(Glyph::new('@')),
            HasFieldOfView(Vec::new()),
            HasInventory(HashSet::new()),
        ));
    }

    fn resolve_command_queue(&mut self, ctx: &mut BTerm) {
        while let Some(comm) = self.commands.pop_front() {
            match comm {
                GameCommand::Flow(FlowCommand::GenerateLevel) => {
                    self.create_new_level()
                },

                GameCommand::Flow(FlowCommand::ReloadViewConfigs) => {
                    self.register_render_view(RenderView::Game);
                    self.register_render_view(RenderView::Debug);
                },

                GameCommand::Flow(FlowCommand::Exit) => ctx.quit(),

                GameCommand::Flow(FlowCommand::CycleViews) => {
                    let view = self.store.get::<RenderView>("view").unwrap_or(RenderView::Game);
                    self.store.set("view", &view.toggle()).unwrap();
                    self.store.set("debug_render_dirty", &true).unwrap();
                },

                _ => {}
            }

            self.dirty = true;
        }
    }
}

impl Game {
    pub fn new(w: i32, h: i32, args: &Opt) -> Game {
        Game { data: GameData::new(w, h, args), }
    }

    pub fn run(args: Opt) {
        println!("{:?}", args);
        rand_gen::init_random_with_seed(args.random_seed);

        if !args.skip_binarize_on_boot {
            println!("Binarizing freshest data...");
            ReadonlyArchiveCave::make_from("resources/data", "resources/data.bin");
        }

        let (width, height) = (80, 50);
        let term = BTermBuilder::new()
            .with_vsync(false)
            .with_title("Poirogue")
            .with_font("classic_roguelike_white.png", 8, 8)
            .with_font("MRMOTEXTEX_rexpaintx2.png", 16, 16)
            .with_font("8x8glyphs.png", 8, 8)

            .with_simple_console(width, height, "classic_roguelike_white.png")
            .with_tile_dimensions(16,16)

            .with_sparse_console(width, height, "MRMOTEXTEX_rexpaintx2.png")
            .with_tile_dimensions(16,16)

            .with_sparse_console(width, height, "8x8glyphs.png")
            .with_tile_dimensions(16,16)

            .build().expect("Couldn't build the terminal window");

        let mut game = Game::new(width, height, &args);

        game.data.setup_store();
        game.data.world.add_unique(NotificationLog::new(3)); // TODO: don't hardcode

        game.data.commands.push_back(GameCommand::Flow(FlowCommand::ReloadViewConfigs));
        game.data.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));

        main_loop(term, game).unwrap();
    }

    fn update_time(&mut self) {
        let mut time = self.data.store.get::<f32>("time").unwrap();
        time += 1.0;
        self.data.store.set::<f32>("time", &time).expect("Failed storing time");
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.update_time();

        self.data.resolve_command_queue(ctx);
        self.data.input.make_new_snapshots(INPUT.lock().borrow());

        let data = &mut self.data;
        let world = &data.world;

        // meta
        world.run_with_data(&core_systems::accept_meta_commands, (&data.input, &mut data.commands)).unwrap();
        world.run_with_data(&core_systems::update_stored_player_position, (&mut data.store));
        world.run_with_data(&core_systems::update_dirty_fovs, (&data.store, &data.map)).unwrap();

        // rendering
        world.run_with_data(&core_systems::update_player_vision, &mut data.map).unwrap();
        world.run_with_data(&core_systems::render_map, (&mut data.map, &mut data.store, ctx)).unwrap();
        world.run_with_data(&game_systems::render_doors, (&mut data.map, ctx)).unwrap();
        world.run_with_data(&game_systems::render_items, (&mut data.map, ctx)).unwrap();
        world.run_with_data(&core_systems::render_characters, (&data.map, ctx)).unwrap();
        world.run_with_data(&game_systems::render_notification_log, ctx).unwrap();

        // cleanup
        world.run(&core_systems::clean_dirty).unwrap();

        // input
        world.run_with_data(&core_systems::interpret_player_input_as_bump_intent, &data.input).unwrap();

        // bump semantics
        world.run(&game_systems::bump__interpret_as_collect_item_intent).unwrap();
        world.run(&game_systems::bump__interpret_as_door_unlock_intent).unwrap();
        world.run_with_data(&game_systems::bump__open_doors, &mut data.map).unwrap();
        world.run_with_data(&game_systems::bump__default, &data.map).unwrap();

        // unlock semantics
        world.run(&game_systems::unlock__if_has_key_for_door).unwrap();
        world.run(&game_systems::unlock__default).unwrap();

        // collect semantics
        world.run(&game_systems::collect__default).unwrap();

        // resolve directives
        world.run_with_data(&game_systems::resolve_move_directives, &data.map).unwrap();
    }
}
