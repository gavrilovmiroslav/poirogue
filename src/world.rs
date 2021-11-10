use std::collections::VecDeque;
use std::sync::{MutexGuard, Mutex};
use lazy_static::lazy_static;

use crate::characters::Character;
use crate::command_queue::CommandQueue;

pub enum GameCommand {
    Load,
    Exit,
}

pub struct World {
    pub commands: VecDeque<GameCommand>,
    pub chars: Vec<Character>,
}

impl CommandQueue for World {
    type CommandType = GameCommand;

    fn get_next(&mut self) -> Option<Self::CommandType> {
        self.commands.pop_front()
    }
}

impl World {
    pub fn new() -> World {
        World { commands: Default::default(), chars: vec![] }
    }
}

lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(World::new());
}

pub fn get_world() -> MutexGuard<'static, World> {
    WORLD.lock().unwrap()
}