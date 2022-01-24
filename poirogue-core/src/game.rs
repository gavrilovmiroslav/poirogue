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
use priority_queue::priority_queue::PriorityQueue;
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
use crate::entity::{HasSight, HasGlyph, HasPosition, IsDirty, IsPlayer, PlayerPosition, Time};
use crate::glyph::Glyph;
use crate::json::InternalJsonStorage;
use crate::opt::Opt;
use crate::rand_gen::{get_random_between, get_random_from, get_random_sub};
use crate::rex::draw_rex;
use crate::tiles::{MapTile, MapTileRep, TileIndex};
use crate::render_view::{RenderViewDefinition};
use crate::render_view::*;
use crate::game_systems;
use crate::game_systems::{BumpIntent, CollectIntent, Intent, IsItem, Item, MoveDirective, NotificationLog, NotifyDirective, ResolvedIntents, UnlockDirective, UnlockIntent};
use crate::maybe::Maybe;

#[derive(Copy, Clone)]
pub struct FlagDebug(pub bool);

#[derive(Copy, Clone)]
pub struct FlagExit(pub bool);

#[derive(Copy, Clone)]
pub struct FlagAnimationDone(pub bool);

pub struct Store(pub(crate) PickleDb);
pub struct MemoryUsageLog(pub(crate) PickleDb);

#[derive(Copy, Clone)]
pub struct WindowSize(pub (i32, i32));

pub struct Binary(Box<dyn Cave>);
pub struct Batch(pub(crate) Reusable<'static, DrawBatch>);

pub struct Game {
    pub debug: bool,
    pub world: World,
    pub args: Opt,
}

pub struct Timeline(PriorityQueue::<Intent, u8>);

impl Timeline {
    pub fn add(&mut self, intent: Intent) {
        let speed = intent.speed;
        self.0.push(intent, speed);
    }

    pub fn next(&mut self) -> Option<Intent> {
        self.0.pop().map(|t| t.0)
    }
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

        game.world.add_unique(FlagDebug(args.keep_memory_log)).expect("Added FlagDebug");
        game.world.add_unique(FlagExit(false)).expect("Added FlagExit");
        game.world.add_unique(FlagAnimationDone(true)).expect("Added FlagAnimationDone");
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

        game.world.add_unique(Batch(DrawBatch::new())).unwrap();
        game.world.add_unique(Time(0u64)).unwrap();
        game.world.add_unique(IsDirty(true)).unwrap();
        game.world.add_unique(PlayerPosition(Point::new(0, 0))).unwrap();
        game.world.add_unique(NotificationLog::new(args.log_height, args.log_expiry)).unwrap();

        game.world.add_unique(ResolvedIntents::default()).unwrap();
        game.world.add_unique(VecDeque::<BumpIntent>::new()).unwrap();
        game.world.add_unique(VecDeque::<UnlockIntent>::new()).unwrap();
        game.world.add_unique(VecDeque::<CollectIntent>::new()).unwrap();
        game.world.add_unique(VecDeque::<MoveDirective>::new()).unwrap();
        game.world.add_unique(VecDeque::<UnlockDirective>::new()).unwrap();
        game.world.add_unique(VecDeque::<NotifyDirective>::new()).unwrap();
        game.world.add_unique(Timeline(PriorityQueue::new()));

        Workload::builder("input handlers")
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
            .with_system(&core_systems::update_fields_of_view)
            .with_system(&core_systems::update_player_position)
            .add_to_world(&game.world).unwrap();

        Workload::builder("player input interpretations")
            .with_system(&core_systems::interpret_player_input_as_bump_intent)
            .with_system(&core_systems::interpret_player_input_as_pickup)
            .add_to_world(&game.world).unwrap();

        Workload::builder("bump interpretations")
//            .with_system(&game_systems::on_bump_interpret_as_collect_item_intent)
            .with_system(&game_systems::on_bump_interpret_as_door_unlock_intent)
            .add_to_world(&game.world).unwrap();

        Workload::builder("item actions")
            .with_system(&game_systems::on_collect_if_possible)
            .add_to_world(&game.world).unwrap();

        Workload::builder("door actions")
            .with_system(&game_systems::on_unlock_if_has_key_for_door)
            .with_system(&game_systems::on_bump_open_doors)
            .add_to_world(&game.world).unwrap();

        Workload::builder("resolve directives")
            .with_system(&game_systems::resolve_unlock_directive)
            .with_system(&game_systems::resolve_move_directives)
            .with_system(&game_systems::resolve_notify_if_entity_alive_directives)
            .add_to_world(&game.world).unwrap();

        Workload::builder("move actions")
            .with_system(&game_systems::on_bump_move_if_empty)
            .add_to_world(&game.world).unwrap();

        Workload::builder("game systems")
            .with_workload("player input interpretations")
            .with_system(&core_systems::push_next_event_from_timeline)
            .with_workload("bump interpretations")
            .with_workload("item actions")
            .with_workload("door actions")
            .with_workload("move actions")
            .with_workload("resolve directives")
            .add_to_world(&game.world).unwrap();

        Workload::builder("render game")
            .with_system(&core_systems::render_player_field_of_view)
            .with_system(&game_systems::render_doors)
            .with_system(&game_systems::render_items)
            .with_system(&core_systems::render_player_visible_characters)
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
