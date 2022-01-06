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
use crate::entity::{HasFieldOfView, HasGlyph, HasPosition, IsDirty, IsPlayer, PlayerPosition, Time};
use crate::glyph::Glyph;
use crate::json::JsonFields;
use crate::opt::Opt;
use crate::rand_gen::{get_random_between, get_random_from, get_random_sub};
use crate::rex::draw_rex;
use crate::tiles::{MapTile, MapTileRep, TileIndex};
use crate::render_view::{RenderViewDefinition};
use crate::render_view::*;
use crate::game_systems;
use crate::game_systems::{BumpIntent, CollectIntent, Handle, InvestigateIntent, IsDoor, IsItem, IsLocked, Item, MoveDirective, NotificationLog, ObjectUsedUp, UnlockDirective, UnlockIntent};

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
    pub usage_log: Option<Store>,
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
            usage_log: if args.keep_memory_log {
                Some(PickleDb::new("memory_usage_log.yaml", PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json))
            } else { None },
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

        game.data.world.add_unique(Time(0u64));
        game.data.world.add_unique(IsDirty(true));
        game.data.world.add_unique(PlayerPosition(Point::new(0, 0)));
        game.data.world.add_unique(NotificationLog::new(args.log_height, args.log_expiry));

        game.data.commands.push_back(GameCommand::Flow(FlowCommand::ReloadViewConfigs));
        game.data.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));

        main_loop(term, game).unwrap();
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.data.resolve_command_queue(ctx);
        self.data.input.make_new_snapshots(INPUT.lock().borrow());

        let data = &mut self.data;
        let world = &data.world;

        // meta
        world.run_with_data(&core_systems::accept_meta_commands, (&data.input, &mut data.commands)).unwrap();

        world.run(&core_systems::update_time).unwrap();
        world.run(&core_systems::update_player_position).unwrap();
        world.run_with_data(&core_systems::update_dirty_fovs, (&data.store, &data.map)).unwrap();
        world.run(&game_systems::update_notification_log_expiry).unwrap();
        world.run_with_data(&core_systems::update_player_vision, &mut data.map).unwrap();

        // rendering game
        ctx.set_active_console(0);
        world.run_with_data(&core_systems::render_map, (&mut data.map, &mut data.store, ctx)).unwrap();
        world.run_with_data(&game_systems::render_doors, (&data.map, ctx)).unwrap();
        world.run_with_data(&game_systems::render_locked_doors, (&data.map, ctx)).unwrap();
        world.run_with_data(&game_systems::render_known_locked_doors, (&data.map, ctx)).unwrap();
        world.run_with_data(&game_systems::render_items, (&data.map, ctx)).unwrap();
        world.run_with_data(&core_systems::render_characters, (&data.map, ctx)).unwrap();

        // rendering gui
        ctx.set_active_console(2);
        ctx.cls();
        world.run_with_data(&game_systems::render_notification_log, ctx).unwrap();
        world.run_with_data(&game_systems::render_fps, ctx).unwrap();

        // cleanup
        world.run(&core_systems::clean_dirty).unwrap();

        // input
        world.run_with_data(&core_systems::interpret_player_input_as_bump_intent, &data.input).unwrap();

        // bump semantics
        world.run(&game_systems::on_bump_interpret_as_collect_item_intent).unwrap();
        world.run(&game_systems::on_bump_interpret_as_door_unlock_intent).unwrap();
        world.run_with_data(&game_systems::on_bump_open_doors, &mut data.map).unwrap();
        world.run_with_data(&game_systems::on_bump_default, &data.map).unwrap();

        // unlock semantics
        world.run(&game_systems::on_unlock_if_has_key_for_door).unwrap();
        world.run(&game_systems::on_unlock_default).unwrap();

        // collect semantics
        world.run(&game_systems::on_collect_default).unwrap();

        // investigate semantics
        world.run(&game_systems::on_investigate_lock).unwrap();
        world.run(&game_systems::on_investigate_default).unwrap();

        // resolve directives
        world.run(&game_systems::resolve_unlock_directive).unwrap();
        world.run_with_data(&game_systems::resolve_move_directives, &data.map).unwrap();

        world.run(&game_systems::delete_handled_intents).unwrap();

        if data.usage_log.is_some() {
            if world.borrow::<UniqueView<Time>>().unwrap().0 % 100 == 0 {
                log_overall_memory_usage(world, &mut data.usage_log.as_mut().unwrap());
            }
        }
    }
}

fn log_memory_usage<T: Send + Sync + 'static>(world: &World, usage_log: &mut Store) {
    if let Ok(view) = world.borrow::<View<T>>() {
        let usage = view.memory_usage().unwrap();
        let name = usage.storage_name.to_string();

        if !usage_log.lexists(name.as_str()) {
            usage_log.lcreate(name.as_str());
            usage_log.lcreate(format!("{}:allocated_memory", name).as_str());
            usage_log.lcreate(format!("{}:used_memory", name).as_str());
            usage_log.lcreate(format!("{}:component_count", name).as_str());
        }

        usage_log.ladd(format!("{}:allocated_memory", name.as_str()).as_str(),  &usage.allocated_memory_bytes);
        usage_log.ladd(format!("{}:used_memory", name.as_str()).as_str(),  &usage.used_memory_bytes);
        usage_log.ladd(format!("{}:component_count", name.as_str()).as_str(),  &usage.component_count);
    }
}

fn log_overall_memory_usage(world: &World, usage_log: &mut Store) {
    log_memory_usage::<Handle<BumpIntent>>(world, usage_log);
    log_memory_usage::<Handle<UnlockIntent>>(world, usage_log);
    log_memory_usage::<Handle<CollectIntent>>(world, usage_log);
    log_memory_usage::<Handle<InvestigateIntent>>(world, usage_log);
    log_memory_usage::<MoveDirective>(world, usage_log);
    log_memory_usage::<UnlockDirective>(world, usage_log);
    log_memory_usage::<IsItem>(world, usage_log);
    log_memory_usage::<IsLocked>(world, usage_log);
    log_memory_usage::<HasPosition>(world, usage_log);
    log_memory_usage::<IsCharacter>(world, usage_log);

    usage_log.dump();
}