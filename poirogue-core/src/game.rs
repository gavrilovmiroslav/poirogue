use std::any::type_name;
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
use shipyard::{AllStoragesViewMut, Storage, StorageMemoryUsage, UniqueView, UniqueViewMut, View, ViewMut, Workload, WorkloadBuilder, World};
use shipyard::error::UniqueRemove::AllStorages;
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
use crate::diagnostics::{create_memory_usage_log, log_overall_memory_usage};
use crate::entity::{HasFieldOfView, HasGlyph, HasPosition, IsDirty, IsPlayer, PlayerPosition, Time};
use crate::glyph::Glyph;
use crate::json::InternalJsonStorage;
use crate::opt::Opt;
use crate::rand_gen::{get_random_between, get_random_from, get_random_sub};
use crate::rex::draw_rex;
use crate::tiles::{MapTile, MapTileRep, TileIndex};
use crate::render_view::{RenderViewDefinition};
use crate::render_view::*;
use crate::game_systems;
use crate::game_systems::{BumpIntent, CollectIntent, Handle, InvestigateIntent, IsDoor, IsItem, IsLocked, Item, MoveDirective, NotificationLog, ObjectUsedUp, on_bump_interpret_as_investigate_intent, UnlockDirective, UnlockIntent};
use crate::maybe::Maybe;

pub type Store = PickleDb;

pub struct Game {
    pub debug: bool,
    pub dirty: bool,
    pub size: (i32, i32),
    pub map: Map,
    pub flow: GameFlow,
    pub commands: VecDeque<GameCommand>,
    pub input: InputSnapshots,
    pub data: Box<dyn Cave>,
    pub store: Store,
    pub world: World,
    pub usage_log: Store,
}

impl InternalJsonStorage for Game {
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

impl Game {
    pub fn new(w: i32, h: i32, args: &Opt) -> Game {
        let data: Box<dyn Cave> = if args.release_mode {
            Box::new(ReadonlyArchiveCave::open(format!("{}.bin", args.data_directory)))
        } else {
            Box::new(FileCave::new(Path::new(args.data_directory.as_str())).unwrap())
        };

        use PickleDbDumpPolicy::*;

        Game {
            debug: args.keep_memory_log,
            dirty: true,
            size: (w, h),
            map: Map::new(w, h),
            flow: GameFlow::Player,
            commands: VecDeque::default(),
            input: InputSnapshots::default(),
            store: PickleDb::new("", PickleDbDumpPolicy::NeverDump, SerializationMethod::Bin),
            world: World::new(),
            usage_log: PickleDb::new(
                if args.keep_memory_log { "memory_usage_log.json" } else { "" },
                if args.keep_memory_log { DumpUponRequest } else { NeverDump },
                SerializationMethod::Json),
            data,
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
    }

    pub fn register_render_view(&self, view: RenderView) {
        fn view_to_file(view: &RenderView) -> &str {
            match view {
                RenderView::Game => "game.view.json",
                RenderView::Debug => "debug.view.json",
            }
        }

        let filename = view_to_file(&view);
        let rep = {
            if let Some(json) = self.get_json::<MapTileRep>(filename) {
                json
            } else {
                self.set_json(filename, &MapTileRep::default());
                MapTileRep::default()
            }
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
                    IsItem{ item: key, is_collected: false },
                    HasPosition(pt),
                    HasGlyph(Glyph::new('(')),
                ));

                self.world.add_component(door_entity, (IsLocked{ key: key_entity },));
                if pt.x % 2 == 0 {
                    self.world.add_component(door_entity, (ObjectUsedUp,));
                }
            }
        }

        let starting_pos = get_random_from(&storage.rects).center();

        self.world.add_entity((
            IsPlayer,
            IsCharacter,
            HasPosition(starting_pos),
            HasGlyph(Glyph::new('@')),
            HasFieldOfView(Vec::new()),
        ));

        self.world.borrow::<UniqueViewMut<IsDirty>>().unwrap().0 = true;
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
                    self.world.borrow::<UniqueViewMut<IsDirty>>().unwrap().0 = true;
                },

                _ => {}
            }

            self.dirty = true;
        }
    }

    pub fn run(args: Opt) {
        println!("{:?}", args);
        rand_gen::init_random_with_seed(args.random_seed);

        if !args.skip_binarize_on_boot {
            println!("Binarizing freshest self...");
            ReadonlyArchiveCave::make_from(
                args.data_directory.as_str(),
                format!("{}.bin", args.data_directory).as_str());
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
        game.setup_store();

        game.world.add_unique(Time(0u64)).unwrap();
        game.world.add_unique(IsDirty(true)).unwrap();
        game.world.add_unique(PlayerPosition(Point::new(0, 0))).unwrap();
        game.world.add_unique(NotificationLog::new(args.log_height, args.log_expiry)).unwrap();

        game.commands.push_back(GameCommand::Flow(FlowCommand::ReloadViewConfigs));
        game.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));

        if game.debug {
            create_memory_usage_log(&mut game.usage_log);
        }

        main_loop(term, game).unwrap();
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.resolve_command_queue(ctx);
        self.input.make_new_snapshots(INPUT.lock().borrow());

        // meta
        self.world.run_with_data(&core_systems::accept_meta_commands, (&self.input, &mut self.commands)).unwrap();

        self.world.run(&core_systems::update_time).unwrap();
        self.world.run(&core_systems::update_player_position).unwrap();
        self.world.run_with_data(&core_systems::update_dirty_fovs, (&self.store, &self.map)).unwrap();
        self.world.run(&game_systems::update_notification_log_expiry).unwrap();
        self.world.run_with_data(&core_systems::update_player_vision, &mut self.map).unwrap();

        // rendering game
        ctx.set_active_console(0);
        self.world.run_with_data(&core_systems::render_map, (&mut self.map, &mut self.store, ctx)).unwrap();
        self.world.run_with_data(&game_systems::render_doors, (&self.map, ctx)).unwrap();
        self.world.run_with_data(&game_systems::render_locked_doors, (&self.map, ctx)).unwrap();
        self.world.run_with_data(&game_systems::render_known_locked_doors, (&self.map, ctx)).unwrap();
        self.world.run_with_data(&game_systems::render_items, (&self.map, ctx)).unwrap();
        self.world.run_with_data(&core_systems::render_characters, (&self.map, ctx)).unwrap();

        // rendering gui
        ctx.set_active_console(2);
        ctx.cls();
        self.world.run_with_data(&game_systems::render_notification_log, ctx).unwrap();
        self.world.run_with_data(&game_systems::render_fps, ctx).unwrap();

        // cleanup
        self.world.run(&core_systems::clean_dirty).unwrap();

        // input
        self.world.run_with_data(&core_systems::interpret_player_input_as_bump_intent, &self.input).unwrap();

        self.world.run(&game_systems::on_bump_interpret_as_collect_item_intent).unwrap();
        self.world.run(&game_systems::on_collect_if_possible).unwrap();
        self.world.run(&game_systems::on_collect_default).unwrap();

        self.world.run(&game_systems::on_bump_interpret_as_door_unlock_intent).unwrap();
        self.world.run(&game_systems::on_unlock_if_has_key_for_door).unwrap();
        self.world.run(&game_systems::on_unlock_default).unwrap();

        self.world.run(&game_systems::on_bump_interpret_as_investigate_intent).unwrap();
        self.world.run(&game_systems::on_investigate_lock).unwrap();
        self.world.run(&game_systems::on_investigate_default).unwrap();

        self.world.run_with_data(&game_systems::on_bump_open_doors, &mut self.map).unwrap();
        self.world.run_with_data(&game_systems::on_bump_default, &self.map).unwrap();

        // resolve directives
        self.world.run(&game_systems::resolve_unlock_directive).unwrap();
        self.world.run_with_data(&game_systems::resolve_move_directives, &self.map).unwrap();

        self.world.run(&game_systems::delete_handled_intents).unwrap();

        if self.debug {
            let time = self.world.borrow::<UniqueView<Time>>().unwrap().0;
            if time % 100 == 0 {
                log_overall_memory_usage(time, &self.world, &mut self.usage_log);
            }
        }
    }
}
