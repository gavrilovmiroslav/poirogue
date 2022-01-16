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
use crate::input::{InputSnapshotState, InputSnapshot, KeyboardSnapshot, InputSnapshots, MouseSnapshot};
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
use crate::game_systems::{BumpIntent, CollectIntent, Handle, InvestigateIntent, IsDoor, IsItem, IsLocked, Item, MoveDirective, NotificationLog, on_bump_interpret_as_investigate_intent, UnlockDirective, UnlockIntent};
use crate::maybe::Maybe;

#[derive(Copy, Clone)]
pub struct FlagDebug(pub bool);

#[derive(Copy, Clone)]
pub struct FlagExit(pub bool);

pub struct Store(pub(crate) PickleDb);
pub struct MemoryUsageLog(pub(crate) PickleDb);

#[derive(Copy, Clone)]
pub struct WindowSize(pub (i32, i32));

pub struct Binary(Box<dyn Cave>);

pub struct Game {
    pub debug: bool,
    pub world: World,
    pub args: Opt,
}


impl Game {
    pub fn new(args: &Opt) -> Game {
        Game {
            debug: args.keep_memory_log,
            world: World::new(),
            args: args.clone(),
        }
    }

    pub fn setup_store(&mut self) {
        let mut store = { self.world.borrow::<UniqueViewMut<Store>>().unwrap() };
        let mut map = { self.world.borrow::<UniqueViewMut<Map>>().unwrap() };
        { /* FOV */
            let fov = 16;
            store.0.set("fov", &fov)
                .expect("Failed storing FOV");
        }

        { /* NOISE MAP */
            let size = (map.width * map.height) as usize;
            let mut noise = Vec::with_capacity(size);
            for _ in 0..size {
                noise.push(get_random_between(0.0, 1.0));
            }

            store.0.set("noise_map", &noise)
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
            if let Some(json) = self.world.borrow::<UniqueView<Binary>>().unwrap().0.get_json::<MapTileRep>(filename) {
                json
            } else {
                self.world.borrow::<UniqueViewMut<Binary>>().unwrap().0.set_json(filename, &MapTileRep::default());
                MapTileRep::default()
            }
        };

        cache_render_view_rep(view, rep);
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

        let mut game = Game::new(&args);

        game.world.add_unique(FlagDebug(args.keep_memory_log)).expect("Added FlagDebug");
        game.world.add_unique(FlagExit(false)).expect("Added FlagExit");
        game.world.add_unique(WindowSize((width, height))).expect("Added WindowSize");
        game.world.add_unique(Map::new(width, height)).expect("Added Map");
        game.world.add_unique(GameFlow::Player).expect("Added GameFlow");

        {
            let mut commands = VecDeque::<GameCommand>::new();
            commands.push_back(GameCommand::Flow(FlowCommand::ReloadViewConfigs));
            commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));
            game.world.add_unique(commands).expect("Added VecDeque<GameCommands>");
        }

        game.world.add_unique(KeyboardSnapshot::default()).expect("Added KeyboardSnapshot");
        game.world.add_unique(MouseSnapshot::default()).expect("Added KeyboardSnapshot");

        {
            game.world.add_unique(Store(PickleDb::new("", PickleDbDumpPolicy::NeverDump, SerializationMethod::Bin)))
                .expect("Added Store");

            game.setup_store();
        }

        {
            use pickledb::PickleDbDumpPolicy::*;

            game.world.add_unique(MemoryUsageLog(PickleDb::new(
                if args.keep_memory_log { "memory_usage_log.json" } else { "" },
                if args.keep_memory_log { DumpUponRequest } else { NeverDump },
                SerializationMethod::Json))).expect("Added MemoryUsageLog");
        }

        game.world.add_unique(Binary(if args.release_mode {
            Box::new(ReadonlyArchiveCave::open(format!("{}.bin", args.data_directory)))
        } else {
            Box::new(FileCave::new(Path::new(args.data_directory.as_str())).unwrap())
        })).expect("Added Binary");

        game.world.add_unique(Time(0u64)).unwrap();
        game.world.add_unique(IsDirty(true)).unwrap();
        game.world.add_unique(PlayerPosition(Point::new(0, 0))).unwrap();
        game.world.add_unique(NotificationLog::new(args.log_height, args.log_expiry)).unwrap();

        if args.keep_memory_log {
            create_memory_usage_log(&mut game.world.borrow::<UniqueViewMut<MemoryUsageLog>>().unwrap());
        }

        main_loop(term, game).unwrap();
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.world.run(&core_systems::make_input_snapshots).unwrap();

        // meta
        self.world.run(&core_systems::on_input_keyboard_exit).unwrap();
        self.world.run(&core_systems::on_input_keyboard_generate_level).unwrap();

        self.world.run(&core_systems::update_time).unwrap();
        self.world.run(&core_systems::update_player_position).unwrap();
        self.world.run(&core_systems::update_dirty_fovs).unwrap();
        self.world.run(&game_systems::update_notification_log_expiry).unwrap();
        self.world.run(&core_systems::update_player_vision).unwrap();

        // rendering game
        ctx.set_active_console(0);
        self.world.run_with_data(&core_systems::render_map, ctx).unwrap();
        self.world.run_with_data(&game_systems::render_doors, ctx).unwrap();
        self.world.run_with_data(&game_systems::render_locked_doors, ctx).unwrap();
        self.world.run_with_data(&game_systems::render_known_locked_doors, ctx).unwrap();
        self.world.run_with_data(&game_systems::render_items, ctx).unwrap();
        self.world.run_with_data(&core_systems::render_characters, ctx).unwrap();

        // rendering gui
        ctx.set_active_console(2);
        ctx.cls();
        self.world.run_with_data(&game_systems::render_notification_log, ctx).unwrap();
        self.world.run_with_data(&game_systems::render_fps, ctx).unwrap();

        // cleanup
        self.world.run(&core_systems::clean_dirty).unwrap();

        // input
        self.world.run(&core_systems::interpret_player_input_as_bump_intent).unwrap();

        self.world.run(&game_systems::on_bump_interpret_as_collect_item_intent).unwrap();
        self.world.run(&game_systems::on_collect_if_possible).unwrap();
        self.world.run(&game_systems::on_collect_default).unwrap();

        self.world.run(&game_systems::on_bump_interpret_as_door_unlock_intent).unwrap();
        self.world.run(&game_systems::on_unlock_if_has_key_for_door).unwrap();
        self.world.run(&game_systems::on_unlock_default).unwrap();

        self.world.run(&game_systems::on_bump_interpret_as_investigate_intent).unwrap();
        self.world.run(&game_systems::on_investigate_lock).unwrap();
        self.world.run(&game_systems::on_investigate_default).unwrap();

        self.world.run(&game_systems::on_bump_open_doors).unwrap();
        self.world.run(&game_systems::on_bump_default).unwrap();

        // resolve directives
        self.world.run(&game_systems::resolve_unlock_directive).unwrap();
        self.world.run(&game_systems::resolve_move_directives).unwrap();

        self.world.run(&game_systems::delete_handled_intents).unwrap();

        if self.debug {
            let time = self.world.borrow::<UniqueView<Time>>().unwrap().0;
            if time % 100 == 0 {
                log_overall_memory_usage(time, &self.world, &mut self.world.borrow::<UniqueViewMut<MemoryUsageLog>>().unwrap());
            }
        }

        if self.world.borrow::<UniqueView<FlagExit>>().unwrap().0 {
            ctx.quit();
        }
    }
}
