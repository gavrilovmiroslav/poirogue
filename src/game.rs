use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use bracket_terminal::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use bracket_lib::prelude::*;
use object_pool::{Pool, Reusable};
use specs::{Builder, Dispatcher, DispatcherBuilder, World, WorldExt};
use crate::map::Map;
use specs::prelude::*;
use crate::commands::{FlowCommand, GameCommand, GameFlow};
use crate::geometry::Glyph;
use crate::input::{InputSnapshotState, InputSnapshot, KeyboardSnapshot, InputSnapshots};
use crate::map_gen::run_map_gen;
use crate::murder_gen::generate_murder;
use crate::render::{OrderedDrawBatch, RenderViewer, RenderView, RenderingPassFn};
use crate::render_view;
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
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.resolve_command_queue(ctx);

        for drawing_func in self.rendering.iter() {
            drawing_func(self).submit();
        }

        render_draw_buffer(ctx);

        self.input.update(INPUT.lock().borrow());
        self.handle_input();
    }
}



impl Game {
    pub fn new(w: i32, h: i32) -> Game {
        Game {
            size: (w, h),
            map: Map::new(w, h),
            flow: GameFlow::Player,
            commands: VecDeque::default(),
            rendering: Vec::new(),
            input: InputSnapshots::default(),
            views: RenderViewer::default(),
        }
    }

    pub fn run() {
        let (width, height) = (80, 50);
        let term = BTermBuilder::new()
            .with_tile_dimensions(16,16)
            .with_font("classic_roguelike_white.png", 8, 8)
            .with_simple_console(width, height, "classic_roguelike_white.png")
            .with_title("Poirogue")
            .build().unwrap();

        let mut game = Game::new(width, height);

        game.views.push(render_view!(DebugView));
        game.views.push(render_view!(GameView));

        game.rendering.push(Box::new(|game| { game.map.render(game.views.get_current_view()) }));
        game.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));

        main_loop(term,game).unwrap();
    }

    pub fn handle_input(&mut self) {
        if self.input.keyboard.is_released(VirtualKeyCode::Escape) {
            self.commands.push_back(GameCommand::Flow(FlowCommand::Exit));
        }

        if self.input.keyboard.is_pressed(VirtualKeyCode::Return) {
            self.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));
        }

        if self.input.mouse.is_pressed(0) {
            self.commands.push_back(GameCommand::Flow(FlowCommand::GenerateLevel));
        }

        if self.input.keyboard.is_pressed(VirtualKeyCode::Tab) {
            self.views.cycle();
        }
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
