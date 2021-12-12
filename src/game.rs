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
use specs::{Builder, Dispatcher, DispatcherBuilder, World, WorldExt};
use crate::map::Map;
use specs::prelude::*;
use crate::readonly_archive_cave::ReadonlyArchiveCave;
use crate::commands::{FlowCommand, GameCommand, GameFlow};
use crate::geometry::Glyph;
use crate::input::{InputSnapshotState, InputSnapshot, KeyboardSnapshot, InputSnapshots};
use crate::map_gen::run_map_gen;
use crate::murder_gen::generate_murder;
use crate::render::{RenderViewGroup, RenderView, RenderingPassFn};
use crate::{Opt, rand_gen, render_view};
use crate::rand_gen::get_random_between;
use crate::rex::draw_rex;
use crate::tiles::MapTile;
use crate::views::{View};
use crate::views_impl::*;


pub struct Entity;

pub struct Game {
    pub shared_data: GameSharedData,
    pub rendering: Vec<RenderingPassFn>,
}

pub struct GameSharedData {
    pub dirty: bool,
    pub size: (i32, i32),
    pub map: Map,
    pub flow: GameFlow,
    pub commands: VecDeque<GameCommand>,
    pub input: InputSnapshots,
    pub data: Box<dyn Cave>,
    pub views: RenderViewGroup<'static>,
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.shared_data.resolve_command_queue(ctx);

        for drawing_func in self.rendering.iter() {
            drawing_func(&mut self.shared_data, ctx);
        }

        self.shared_data.handle_input();
        self.shared_data.dirty = false;
    }
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
            views: RenderViewGroup::default(),
            data: if args.release_mode {
                println!("Loading data from binarized file...");
                Box::new(ReadonlyArchiveCave::open("resources/data.bin"))
            } else {
                println!("Loading data from resource folder...");
                Box::new(FileCave::new(Path::new("resources/data")).unwrap())
            },
        }
    }

    pub fn handle_input(&mut self) {
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
        }

        self.input.update(INPUT.lock().borrow());
        handle_editor_input(self);
    }

    fn resolve_command_queue(&mut self, ctx: &mut BTerm) {
        while let Some(comm) = self.commands.pop_front() {
            match comm {
                GameCommand::Flow(FlowCommand::GenerateLevel) => {
                    self.map = run_map_gen(self.size.0, self.size.1);
                },

                GameCommand::Flow(FlowCommand::Exit) => ctx.quit(),

                GameCommand::Flow(FlowCommand::CycleViews) => {
                    self.views.cycle();
                },
            }

            self.dirty = true;
        }
    }
}

impl Game {
    pub fn new(w: i32, h: i32, args: &Opt) -> Game {
        Game {
            shared_data: GameSharedData::new(w, h, args),
            rendering: Vec::new(),
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

            .build().unwrap();

        let mut game = Game::new(width, height, &args);

        game.shared_data.views.push(render_view!(DebugView));
        game.shared_data.views.push(render_view!(GameView));

        game.rendering.push(Box::new(|game, ctx| {
            if game.dirty {
                ctx.set_active_console(0);
                ctx.cls();
                game.map.render(ctx, game.views.get_current_view());
            }
        }));

        game.rendering.push(Box::new(|game, ctx| {
            ctx.set_active_console(1);
            ctx.cls();
            let pos = ctx.mouse_pos();
            draw_rex(game, ctx,"skull", pos.0 + 2, pos.1 + 2);

            ctx.set_active_console(2);
            ctx.cls();
            ctx.print_color(1, 1, RGB::named(WHITE), RGB::named(BLACK), format!("FPS: {}", ctx.fps));
        }));

        game.shared_data.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));

        main_loop(term,game).unwrap();
    }
}
