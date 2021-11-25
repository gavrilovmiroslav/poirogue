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
use crate::geometry::Glyph;
use crate::input::{InputSnapshotState, InputSnapshot, KeyboardSnapshot, InputSnapshots};
use crate::map_gen::run_map_gen;
use crate::murder_gen::generate_murder;

pub trait RenderBrain {
    fn render(&self) -> OrderedDrawBatch;
}

pub enum FlowCommand {
    GenerateLevel,
    Exit
}

pub enum GameCommand {
    Flow(FlowCommand),
}

pub enum GameFlow {
    Player,
    World,
}

pub struct OrderedDrawBatch(usize, Reusable<'static, DrawBatch>);

impl OrderedDrawBatch {
    pub fn new(z: usize, db: Reusable<'static, DrawBatch>) -> OrderedDrawBatch {
        OrderedDrawBatch(z, db)
    }

    pub fn submit(&mut self) {
        self.1.submit(self.0).expect("Batch error");
    }
}

#[derive(Default)]
pub struct FrameData {
    batches: Vec<OrderedDrawBatch>
}

pub struct Game {
    pub size: (i32, i32),
    pub map: Map,
    pub flow: GameFlow,
    pub commands: VecDeque<GameCommand>,
    pub rendering: Vec<Box<dyn Fn(&Game) -> OrderedDrawBatch>>,
    pub input: InputSnapshots,
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut frame_data = FrameData::default();

        self.resolve_command_queue(ctx);

        self.queue_frame_rendering(&mut frame_data);
        for mut batch in frame_data.batches {
            batch.submit();
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

        game.rendering.push(Box::new(|game| game.map.render() ));
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
    }

    pub fn queue_frame_rendering(&mut self, frame_data: &mut FrameData) {
        for brain in self.rendering.iter() {
            frame_data.batches.push(brain(self));
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
