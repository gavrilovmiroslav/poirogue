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
use crate::render::{RenderViewer, RenderView, RenderingPassFn};
use crate::{Opt, rand_gen, render_view};
use crate::rex::draw_rex;
use crate::tiles::MapTile;
use crate::views::{View};
use crate::views_impl::*;


pub struct Entity;

pub struct Game {
    pub size: (i32, i32),
    pub map: Map,
    pub flow: GameFlow,
    pub commands: VecDeque<GameCommand>,
    pub rendering: Vec<RenderingPassFn>,
    pub input: InputSnapshots,
    pub views: RenderViewer<'static>,
    pub data: Box<dyn Cave>,
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.resolve_command_queue(ctx);

        for drawing_func in self.rendering.iter() {
            drawing_func(self, ctx);
        }

        let xy = ctx.mouse_pos();

        self.input.update(INPUT.lock().borrow());
        self.handle_input();
    }
}

impl Game {
    pub fn new(w: i32, h: i32, args: &Opt) -> Game {
        Game {
            size: (w, h),
            map: Map::new(w, h),
            flow: GameFlow::Player,
            commands: VecDeque::default(),
            rendering: Vec::new(),
            input: InputSnapshots::default(),
            views: RenderViewer::default(),
            data: if args.release_mode {
                println!("Loading data from binarized file...");
                Box::new(ReadonlyArchiveCave::open("resources/data.bin"))
            } else {
                println!("Loading data from resource folder...");
                Box::new(FileCave::new(Path::new("resources/data")).unwrap())
            },
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
            .with_title("Poirogue")
            .with_font("classic_roguelike_white.png", 8, 8)
            .with_font("MRMOTEXTEX_rexpaintx2.png", 16, 16)

            .with_simple_console(width, height, "classic_roguelike_white.png")
            .with_tile_dimensions(16,16)

            .with_sparse_console(width, height, "MRMOTEXTEX_rexpaintx2.png")
            .with_tile_dimensions(16,16)
            .build().unwrap();

        let mut game = Game::new(width, height, &args);

        game.views.push(render_view!(DebugView));
        game.views.push(render_view!(GameView));

        game.rendering.push(Box::new(|game, ctx| {
            ctx.set_active_console(0);
            game.map.render(ctx,game.views.get_current_view());
        }));

        game.rendering.push(Box::new(|game, ctx| {
            ctx.set_active_console(1);
            ctx.cls();
            let mouse = ctx.mouse_pos();
            let data = game.data.borrow();
            draw_rex(data, ctx, "frame", mouse.0 + 1, mouse.1 + 1);
            draw_rex(data, ctx,"skull", mouse.0 + 2, mouse.1 + 2);
            draw_rex(data, ctx, "frame", mouse.0 + 9, mouse.1 + 9);
            draw_rex(data, ctx,"skull", mouse.0 + 10, mouse.1 + 10);
            draw_rex(data, ctx, "frame", mouse.0 + 9, mouse.1 + 1);
            draw_rex(data, ctx,"skull", mouse.0 + 10, mouse.1 + 2);
            draw_rex(data, ctx, "frame", mouse.0 + 1, mouse.1 + 9);
            draw_rex(data, ctx,"skull", mouse.0 + 2, mouse.1 + 10);
        }));

        game.rendering.push(Box::new(|game, ctx| {
            ctx.set_active_console(0);
            ctx.print_color(2, 2, RGB::named(WHITE), RGB::named(BLACK), format!(" FPS: {} ", ctx.fps));
        }));

        game.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));

        main_loop(term,game).unwrap();
    }

    pub fn handle_input(&mut self) {
        fn handle_editor_input(game: &mut Game) {
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
                game.views.cycle();
            }
        }

        handle_editor_input(self);
    }

    fn resolve_command_queue(&mut self, ctx: &mut BTerm) {
        while let Some(comm) = self.commands.pop_front() {
            match comm {
                GameCommand::Flow(FlowCommand::GenerateLevel) => {
                    self.map = run_map_gen(self.size.0, self.size.1);
                },

                GameCommand::Flow(FlowCommand::Exit) => ctx.quit()
            }
        }
    }
}
