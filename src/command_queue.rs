use std::collections::VecDeque;
use std::sync::mpsc::{channel, Sender, Receiver};
use crate::world::GameCommand;

pub struct Queueable<Commands> {
    commands: VecDeque<Commands>,
}

impl<T> Default for Queueable<T> {
    fn default() -> Self {
        Queueable { commands: Default::default() }
    }
}

impl<CommandType> Queueable<CommandType> {
    pub fn push(&mut self, comm: CommandType) {
        self.commands.push_back(comm);
    }

    pub fn get_next(&mut self) -> Option<CommandType> {
        self.commands.pop_front()
    }
}