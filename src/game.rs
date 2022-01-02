use std::str;
use std::borrow::{Borrow, BorrowMut};
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
use crate::commands::{ActionCommand, FlowCommand, GameCommand, GameFlow, HackCommand};
use crate::input::{InputSnapshotState, InputSnapshot, KeyboardSnapshot, InputSnapshots};
use crate::map_gen::run_map_gen;
use crate::murder_gen::generate_murder;
use crate::{rand_gen};
use crate::colors::named_color;
use crate::entity::{AbstractEntity, Entity};
use crate::opt::Opt;
use crate::rand_gen::get_random_between;
use crate::rex::draw_rex;
use crate::tiles::{DoorState, MapTile, MapTileRep};
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
            self.data.set(name, &json.as_bytes().to_vec()).unwrap();
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

            if game.input.keyboard.is_pressed(VirtualKeyCode::Tab) {
                game.commands.push_back(GameCommand::Flow(FlowCommand::CycleViews));
            }

            if game.input.keyboard.is_pressed(VirtualKeyCode::F2) {
                game.commands.push_back(GameCommand::Hack(HackCommand::UnlockAllDoors));
            }

            if game.input.keyboard.is_pressed(VirtualKeyCode::F3) {
                game.commands.push_back(GameCommand::Hack(HackCommand::LockAllDoors));
            }

            if game.input.keyboard.is_pressed(VirtualKeyCode::F4) {
                game.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));
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

        let mut noise = Vec::with_capacity((width * height) as usize);
        for i in 0..width * height {
            noise.push(get_random_between(0.0f32, 1.0f32));
        }

        { /* FOV */
            let fov = 32;
            game.shared_data.store.set("fov", &fov);
        }

        {
            /* NOISE MAP */
            game.shared_data.store.set("noise_map", &noise);
        }

        fn render_map_if_dirty_and_view(view: &RenderView, game: &GameSharedData, ctx: &mut BTerm) {
            if game.dirty {
                if *view == game.store.get::<RenderView>("view").unwrap_or(RenderView::Game) {
                    ctx.set_active_console(0);
                    ctx.cls();
                    game.map.render(ctx, view, &game.store);
                }

                for holder in &game.entities {
                    if let cell = holder.clone().as_ref() {
                        let entity = cell.borrow();
                        let pos = entity.get_position();
                        let glyph = entity.get_glyph();
                        let index = game.map.point2d_to_index(pos);

                        if game.map.is_tile_revealed(index) && game.map.visible[index] {
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
            ctx.print_color(1, 1, named_color(WHITE), named_color(BLACK), format!("FPS: {}", ctx.fps));
        }));

        game.shared_data.commands.push_back(GameCommand::Flow(FlowCommand::ReloadViewConfigs));
        game.shared_data.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));

        main_loop(term,game).unwrap();
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        { /* TIME */
            let mut time = self.shared_data.store.get::<f32>("time").unwrap_or(0.0);
            time += 0.03;
            self.shared_data.store.set::<f32>("time", &time);
        }

        self.shared_data.resolve_command_queue(ctx);

        for drawing_func in self.render_pipeline.iter() {
            drawing_func(&mut self.shared_data, ctx);
        }

        self.shared_data.collect_inputs();

        for holder in &self.shared_data.entities {
            if let cell = holder.clone().as_ref() {
                let changes = cell.borrow().tick(&self.shared_data);

                for change in changes {
                    match change {
                        ActionCommand::MoveBy(dx, dy) => {
                            let mut e = cell.borrow_mut();
                            let new_pos = e.inner().position + Point::new(dx, dy);
                            let map = &self.shared_data.map;
                            let index = map.point2d_to_index(new_pos);
                            if !map.is_tile_blocked(index) {
                                e.inner_mut().position = new_pos;
                            } else {
                                if let MapTile::Door(DoorState::Closed) = map.tiles[index] {
                                    self.shared_data.map.set(new_pos.x, new_pos.y, MapTile::Door(DoorState::Open));
                                }
                            }
                        },

                        ActionCommand::MoveTo(x, y) => {
                            let mut e = cell.borrow_mut();
                            e.inner_mut().position.x = x;
                            e.inner_mut().position.y = y;
                        }

                        ActionCommand::FovChange(fov) => {
                            let mut map = &mut self.shared_data.map;

                            map.hide();
                            map.show(fov);
                        }
                    }
                }

                {
                    let entity = cell.borrow();
                    if entity.is_player() {
                        let pt = entity.get_position();
                        let p = (pt.x, pt.y);
                        self.shared_data.store.set("player_position", &p);
                    }
                }
            }
        }
    }
}
