use std::str;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
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

use crate::map::Map;
use crate::readonly_archive_cave::ReadonlyArchiveCave;
use crate::commands::{ActionCommand, FlowCommand, GameCommand, GameFlow, HackCommand};
use crate::input::{InputSnapshotState, InputSnapshot, KeyboardSnapshot, InputSnapshots};
use crate::map_gen::run_map_gen;
use crate::murder_gen::generate_murder;
use crate::{core_systems, POSITION_QUERY_REQUEST_QUEUE, rand_gen};
use crate::colors::named_color;
use crate::entity::{HasFieldOfView, HasGlyph, HasPosition, IsDirty, IsPlayer};
use crate::glyph::Glyph;
use crate::json::JsonFields;
//use crate::entity::{AbstractEntity, Entity};
use crate::opt::Opt;
use crate::rand_gen::{get_random_between, get_random_from};
use crate::rex::draw_rex;
use crate::tiles::{DoorState, MapTile, MapTileRep};
use crate::render_view::{View};
use crate::render_view::*;
use crate::store_helpers::StoreHelpers;

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

        { /* STORED LISTS */
            self.store.lcreate(POSITION_QUERY_REQUEST_QUEUE);
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
        self.store.lregen(POSITION_QUERY_REQUEST_QUEUE);

        let (map, storage) = run_map_gen(self.size.0, self.size.1);
        self.map = map;

        let random_center_point = get_random_from(&storage.rects).center();
        self.world.add_entity((
            IsPlayer,
            IsDirty(true),
            HasPosition(random_center_point),
            HasGlyph(Glyph::new('@')),
            HasFieldOfView(Vec::new()),
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
                    self.store.set::<RenderView>("view", &view.toggle()).expect("Store entry for 'view' updated successfully");
                },

                GameCommand::Hack(HackCommand::UnlockAllDoors) => {
                    self.map.iter_all_doors(&Map::open);
                },

                GameCommand::Hack(HackCommand::LockAllDoors) => {
                    self.map.iter_all_doors(&Map::close);
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

        game.data.commands.push_back(GameCommand::Flow(FlowCommand::ReloadViewConfigs));
        game.data.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));

        main_loop(term,game).unwrap();
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

        // META STUFF

        world.run_with_data(&core_systems::accept_meta_commands, (&data.input, &mut data.commands)).unwrap();

        // ONE FRAME (UPDATING ALL DIRTY STUFF)

        world.run_with_data(&core_systems::update_stored_player_position, (&mut data.store));
        world.run_with_data(&core_systems::update_dirty_fovs, (&data.store, &data.map)).unwrap();
        world.run_with_data(&core_systems::update_revealed_map, &mut data.map).unwrap();
        world.run_with_data(&core_systems::render_map, (&mut data.map, &data.store, ctx)).unwrap();
        world.run_with_data(&core_systems::render_entities, (&data.map, ctx)).unwrap();
        world.run(&core_systems::clean_dirty).unwrap();

        // PREP FOR SECOND FRAME (ALL DIRTY IS CLEAN HERE)

        world.run_with_data(&core_systems::handle_player_controls, (&data.input, &mut data.store)).unwrap();
        world.run_with_data(&core_systems::query_position_for_wall_blocking, (&data.map, &mut data.store)).unwrap();
        world.run_with_data(&core_systems::query_position_for_door_opening, (&mut data.map, &mut data.store)).unwrap();
        world.run_with_data(&core_systems::handle_move_to_commands, &data.map).unwrap();
    }
}
