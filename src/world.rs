use std::collections::VecDeque;
use std::sync::{MutexGuard, Mutex};
use lazy_static::lazy_static;
use crate::command_queue::Queueable;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Receiver, RecvError, Sender, TryRecvError};
use std::thread;
use std::thread::{JoinHandle, Thread};
use bracket_lib::prelude::BTerm;
use crate::floor_builder::FloorMapBuilder;
use crate::map::PoirogueMap;
use crate::murder_gen::{generate_murder};

use crate::pawn::Pawn;

pub enum GameCommand {
    Gen,
    Exit,
}

pub trait Tick {
    fn tick(&mut self, ctx: &mut BTerm);
}

pub struct World {
    pub commands: Queueable<GameCommand>,
    pub receiver: Receiver<GameCommand>,
    pub sender: Sender<GameCommand>,
    pub map: PoirogueMap,
}

impl World {
    pub fn new() -> World {
        let (s, r) = channel::<GameCommand>();
        World {
            commands: Default::default(),
            receiver: r,
            sender: s,
            map: PoirogueMap::new(80, 50),
        }
    }

    pub fn receive_commands(&mut self) {
        if let Ok(message) = self.receiver.try_recv() {
            match message {
                GameCommand::Gen => {
                    let mut floor = FloorMapBuilder::new();
                    let map = &mut self.map;
                    floor.generate(map);

                    let (case, rels) = generate_murder();
                },
                GameCommand::Exit => std::process::exit(0)
            }
        }
    }

    pub fn update(&mut self, ctx: &mut BTerm) {
        for pawn in &mut self.map.entities {
            pawn.update(ctx);
        }
    }
}

lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(World::new());
}

pub fn get_world() -> MutexGuard<'static, World> {
    WORLD.lock().unwrap()
}