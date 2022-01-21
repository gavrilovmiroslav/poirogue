use std::any::type_name;
use std::str;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::collections::hash_map::RandomState;
use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use bracket_terminal::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use bracket_color::prelude::RGB;
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
use crate::{core_systems, DRAWING_CONSOLE_LAYER, MAP_CONSOLE_LAYER, rand_gen, UI_CONSOLE_LAYER};
use crate::colors::named_color;
use crate::entity::{HasFieldOfView, HasGlyph, HasPosition, IsDirty, IsPlayer, PlayerPosition, Time};
use crate::glyph::{Glyph, GlyphMap, register_glyphs};
use crate::json::InternalJsonStorage;
use crate::opt::Opt;
use crate::rand_gen::{get_random_between, get_random_from, get_random_sub};
use crate::rex::draw_rex;
use crate::tiles::{MapTile, TileIndex};
use crate::game_systems;
use crate::game_systems::{BumpIntent, CollectIntent, InvestigateIntent, IsDoor, IsItem, IsLocked, Item, MoveDirective, NotificationLog, on_bump_interpret_as_investigate_intent, ResolvedIntents, UnlockDirective, UnlockIntent};
use crate::maybe::Maybe;
use crate::colors::*;

#[derive(Copy, Clone)]
pub struct FlagDebug(pub bool);

#[derive(Copy, Clone)]
pub struct FlagRecompileScripts(pub bool);

#[derive(Copy, Clone)]
pub struct FlagExit(pub bool);

pub struct Store(pub(crate) PickleDb);
pub struct MemoryUsageLog(pub(crate) PickleDb);

#[derive(Copy, Clone)]
pub struct WindowSize(pub (i32, i32));

pub struct BinaryData(pub Box<dyn Cave>);
pub struct Batch(pub Reusable<'static, DrawBatch>);
pub struct Scripting(pub rhai::Engine);

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
            .with_fps_cap(200.0)
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

        game.world.add_unique(FlagRecompileScripts(true)).expect("Added FlagRecompileScripts");
        game.world.add_unique(FlagDebug(args.keep_memory_log)).expect("Added FlagDebug");
        game.world.add_unique(FlagExit(false)).expect("Added FlagExit");
        game.world.add_unique(WindowSize((width, height))).expect("Added WindowSize");
        game.world.add_unique(Map::new(width, height)).expect("Added Map");
        game.world.add_unique(GlyphMap::new(width, height)).expect("Added GlyphMap");
        game.world.add_unique(GameFlow::Player).expect("Added GameFlow");

        {
            let engine = {
                use crate::tiles::*;
                let mut engine = rhai::Engine::new();

                engine.register_type_with_name::<MapTile>("MapTile")
                    .register_fn("name", MapTile::name);

                engine.register_type_with_name::<RGB>("RGB")
                    .register_fn("make_rgb", |r: i64, g: i64, b: i64| RGB::from((r as u8, g as u8, b as u8)))
                    .register_fn("*", |a: RGB, b: RGB| a * b);

                engine.register_type_with_name::<Point>("Point")
                    .register_fn("get_x", |p: Point| p.x as i64)
                    .register_fn("get_y", |p: Point| p.y as i64);

                register_glyphs(&mut engine);

                engine
            };

            game.world.add_unique(Scripting(engine));
        }

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

        game.world.add_unique(BinaryData(if args.release_mode {
            Box::new(ReadonlyArchiveCave::open(format!("{}.bin", args.data_directory)))
        } else {
            Box::new(FileCave::new(Path::new(args.data_directory.as_str())).unwrap())
        })).expect("Added Binary");

        game.world.add_unique(Batch(DrawBatch::new())).unwrap();
        game.world.add_unique(Time(0u64)).unwrap();
        game.world.add_unique(IsDirty(true)).unwrap();
        game.world.add_unique(PlayerPosition(Point::new(0, 0))).unwrap();
        game.world.add_unique(NotificationLog::new(args.log_height, args.log_expiry)).unwrap();

        game.world.add_unique(ResolvedIntents::default()).unwrap();
        game.world.add_unique(VecDeque::<BumpIntent>::new()).unwrap();
        game.world.add_unique(VecDeque::<UnlockIntent>::new()).unwrap();
        game.world.add_unique(VecDeque::<CollectIntent>::new()).unwrap();
        game.world.add_unique(VecDeque::<InvestigateIntent>::new()).unwrap();
        game.world.add_unique(VecDeque::<MoveDirective>::new()).unwrap();
        game.world.add_unique(VecDeque::<UnlockDirective>::new()).unwrap();

        Workload::builder("input handlers")
            .with_system(&game_systems::on_input_mark_recompile_scripts)
            .with_system(&game_systems::on_input_keyboard_exit)
            .with_system(&game_systems::on_input_keyboard_generate_level)
            .add_to_world(&game.world).unwrap();

        Workload::builder("game command interpretations")
            .with_system(&core_systems::on_command_generate_level)
            .add_to_world(&game.world).unwrap();

        Workload::builder("realtime updates")
            .with_system(&game_systems::update_notification_log_expiry)
            .add_to_world(&game.world).unwrap();

        Workload::builder("dirty-only updates")
            .with_system(&core_systems::update_player_position)
            .with_system(&core_systems::update_player_vision)
            .add_to_world(&game.world).unwrap();

        Workload::builder("player input interpretations")
            .with_system(&core_systems::interpret_player_input_as_bump_intent)
            .add_to_world(&game.world).unwrap();

        Workload::builder("bump interpretations")
            .with_system(&game_systems::on_bump_interpret_as_collect_item_intent)
            .with_system(&game_systems::on_bump_interpret_as_door_unlock_intent)
            .with_system(&game_systems::on_bump_interpret_as_investigate_intent)
            .add_to_world(&game.world).unwrap();

        Workload::builder("item actions")
            .with_system(&game_systems::on_collect_if_possible)
            .add_to_world(&game.world).unwrap();

        Workload::builder("door actions")
            .with_system(&game_systems::on_unlock_if_has_key_for_door)
            .with_system(&game_systems::on_investigate_lock)
            .with_system(&game_systems::on_bump_open_doors)
            .add_to_world(&game.world).unwrap();

        Workload::builder("resolve directives")
            .with_system(&game_systems::resolve_unlock_directive)
            .with_system(&game_systems::resolve_move_directives)
            .add_to_world(&game.world).unwrap();

        Workload::builder("move actions")
            .with_system(&game_systems::on_bump_move_if_empty)
            .add_to_world(&game.world).unwrap();

        Workload::builder("game systems")
            .with_workload("player input interpretations")
            .with_workload("bump interpretations")
            .with_workload("item actions")
            .with_workload("door actions")
            .with_workload("move actions")
            .with_workload("resolve directives")
            .add_to_world(&game.world).unwrap();

        Workload::builder("render game")
            .with_system(&core_systems::render_player_field_of_view)
            .with_system(&game_systems::render_doors)
            .with_system(&game_systems::render_locked_doors)
            .with_system(&game_systems::render_known_locked_doors)
            .with_system(&core_systems::render_player_visible_characters)
            .with_system(&game_systems::render_items)
            .add_to_world(&game.world).unwrap();

        main_loop(term, game).unwrap();
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        fn process_game_commands(game: &mut Game) {
            let mut more_commands = true;

            while more_commands {
                let command_count = game.world.borrow::<UniqueView<VecDeque<GameCommand>>>().unwrap().len();
                game.world.run_workload("game command interpretations").unwrap();
                let new_command_count = game.world.borrow::<UniqueView<VecDeque<GameCommand>>>().unwrap().len();

                if command_count == new_command_count && command_count > 0 {
                    println!("Warning: removing uninterpreted game command {:?}",
                             *game.world.borrow::<UniqueView<VecDeque<GameCommand>>>().unwrap().front().unwrap());

                    game.world.borrow::<UniqueViewMut<VecDeque<GameCommand>>>().unwrap().pop_front();
                    more_commands = false;
                } else if new_command_count == 0 {
                    more_commands = false;
                }
            }
        }

        // meta
        self.world.run(&game_systems::make_input_snapshots).unwrap();
        self.world.run_workload("input handlers").unwrap();

        process_game_commands(self);

        self.world.run(&core_systems::update_time).unwrap();
        self.world.run_workload("realtime updates").unwrap();

        self.world.run(&core_systems::clear_ast_lru_cache_if_requested).unwrap();

        if self.world.borrow::<UniqueView<IsDirty>>().unwrap().0 {
            ctx.set_active_console(MAP_CONSOLE_LAYER); ctx.cls();
            ctx.set_active_console(DRAWING_CONSOLE_LAYER); ctx.cls();
            self.world.run_workload("dirty-only updates").unwrap();

            { self.world.borrow::<UniqueViewMut<Batch>>().unwrap().0.cls(); }

            self.world.run_workload("render game").unwrap();
            self.world.run_with_data(&core_systems::submit_draw_batching, ctx).unwrap();
        }

        ctx.set_active_console(UI_CONSOLE_LAYER); ctx.cls();
        self.world.run_with_data(&game_systems::render_fps, ctx).unwrap();
        self.world.run_with_data(&game_systems::render_notification_log, ctx).unwrap();

        // cleanup
        self.world.run(&core_systems::clean_dirty).unwrap();
        self.world.run_workload("game systems").unwrap();
        self.world.run(&game_systems::delete_intents).unwrap();

        if self.world.borrow::<UniqueView<FlagExit>>().unwrap().0 {
            ctx.quit();
        }
    }
}
