use std::str;
use std::borrow::{Borrow};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::rc::{Rc, Weak};
use bracket_terminal::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use bracket_lib::prelude::*;
use caves::{Cave, FileCave, MemoryCave};
use object_pool::{Pool, Reusable};
use planck_ecs::{Dispatcher, DispatcherBuilder, World};
use lazy_static::*;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serde::{de, Serialize};

use crate::map::Map;
use crate::readonly_archive_cave::ReadonlyArchiveCave;
use crate::commands::{FlowCommand, GameCommand, GameFlow};
use crate::input::{InputSnapshotState, InputSnapshot, KeyboardSnapshot, InputSnapshots};
use crate::map_gen::run_map_gen;
use crate::murder_gen::generate_murder;
use crate::{rand_gen};
use crate::entity::{AbstractEntity, Entity};
use crate::opt::Opt;
use crate::rand_gen::get_random_between;
use crate::rex::draw_rex;
use crate::tiles::{MapTile, MapTileRep};
use crate::render_view::{View};
use crate::render_view::*;

pub type RenderingPassFn = Box<dyn Fn(&mut GameSharedData, &mut BTerm)>;
pub type Store = PickleDb;

pub struct Game {
    pub shared_data: GameSharedData,
    pub render_pipeline: Vec<RenderingPassFn>,
}

pub struct GameSharedData {
    pub dirty: bool,
    pub size: (i32, i32),
    pub map: Map,
    pub flow: GameFlow,
    pub commands: VecDeque<GameCommand>,
    pub input: InputSnapshots,
    pub data: Box<dyn Cave>,
    pub entities: Vec<Rc<RefCell<dyn AbstractEntity<Data = GameSharedData>>>>,
    pub store: Store,
}

impl GameSharedData {
    pub fn new(w: i32, h: i32, args: &Opt) -> GameSharedData {
        GameSharedData {
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
            entities: Vec::default(),
            store: PickleDb::new("", PickleDbDumpPolicy::NeverDump, SerializationMethod::Bin),
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

        add_render_view_rep(view, rep);
    }

    pub fn get_json<T>(&self, name: &str) -> Option<T>
    where for <'a> T: de::Deserialize<'a> {
        if let Ok(binary_data) = self.data.get(name) {
            if let Ok(string_data) = str::from_utf8(&binary_data) {
                let json_as_struct = serde_json::from_str::<T>(string_data).unwrap();
                return Some(json_as_struct)
            }
        }

        return None;
    }

    pub fn set_json<T>(&self, name: &str, value: &T)
        where T: ?Sized + Serialize {

        if let Ok(json) = serde_json::to_string_pretty(value) {
            self.data.set(name, &json.as_bytes().to_vec());
        }
    }


    pub fn collect_inputs(&mut self) {
        fn handle_editor_input(game: &mut GameSharedData) {
            if game.input.keyboard.is_released(VirtualKeyCode::Escape) {
                game.commands.push_back(GameCommand::Flow(FlowCommand::Exit));
            }

            if game.input.keyboard.is_pressed(VirtualKeyCode::Return) {
                game.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));
            }

            if game.input.mouse.is_pressed(0) {
                game.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));
            }

            if game.input.keyboard.is_pressed(VirtualKeyCode::Tab) {
                game.commands.push_back(GameCommand::Flow(FlowCommand::CycleViews));
            }

            if game.input.keyboard.is_pressed(VirtualKeyCode::F5) {
                game.commands.push_back(GameCommand::Flow(FlowCommand::ReloadViewConfigs));
            }
        }

        self.input.make_new_snapshots(INPUT.lock().borrow());
        handle_editor_input(self);
    }

    fn create_new_level(&mut self) {
        let (map, storage) = run_map_gen(self.size.0, self.size.1);
        self.map = map;

        self.entities.clear();
        let player = Entity::make_player(storage.rects[0].center());
        self.entities.push(player);

        for i in 1..10 {
            let character = Entity::make_character(storage.rects[i].center(), 'K');
            self.entities.push(character);
        }
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
            }

            self.dirty = true;
        }
    }
}

impl Game {
    pub fn new(w: i32, h: i32, args: &Opt) -> Game {
        Game {
            render_pipeline: Vec::new(),
            shared_data: GameSharedData::new(w, h, args)
        }
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

        fn render_map_if_dirty_and_view(view: &RenderView, game: &GameSharedData, ctx: &mut BTerm) {
            if game.dirty {
                if *view == game.store.get::<RenderView>("view").unwrap_or(RenderView::Game) {
                    ctx.set_active_console(0);
                    ctx.cls();
                    game.map.render(ctx, view);
                }

                for holder in &game.entities {
                    if let cell = holder.clone().as_ref() {
                        let entity = cell.borrow();
                        let pos = entity.get_position();
                        let glyph = entity.get_glyph();

                        if game.map.is_tile_revealed(game.map.point2d_to_index(pos)) {
                            ctx.print_color(pos.x, pos.y, RGB::from(glyph.fg), RGB::from(glyph.bg), glyph.ch);
                        }
                    }
                }
            }
        }

        // game view
        game.render_pipeline.push(Box::new(|game, ctx| {
            render_map_if_dirty_and_view(&RenderView::Game, game, ctx);
        }));

        // debug view
        game.render_pipeline.push(Box::new(|game, ctx| {
            render_map_if_dirty_and_view(&RenderView::Debug, game, ctx);
        }));

        // gui
        game.render_pipeline.push(Box::new(|game, ctx| {
            ctx.set_active_console(2);
            ctx.cls();
            ctx.print_color(1, 1, RGB::named(WHITE), RGB::named(BLACK), format!("FPS: {}", ctx.fps));
        }));

        game.shared_data.commands.push_back(GameCommand::Flow(FlowCommand::ReloadViewConfigs));
        game.shared_data.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));

        main_loop(term,game).unwrap();
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.shared_data.resolve_command_queue(ctx);

        for drawing_func in self.render_pipeline.iter() {
            drawing_func(&mut self.shared_data, ctx);
        }

        self.shared_data.collect_inputs();

        for holder in &self.shared_data.entities {
            if let cell = holder.clone().as_ref() {
                let mut entity = cell.borrow_mut();
                entity.tick(&self.shared_data);

                if entity.is_player() && entity.is_dirty() {
                    let mut map = &mut self.shared_data.map;

                    map.hide();
                    map.show(entity.get_fov().iter());

                    self.shared_data.dirty = true;
                }
            }
        }
    }
}
