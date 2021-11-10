use std::collections::VecDeque;
use bracket_lib::prelude::{Point, RGB};
use crate::command_queue::{CommandInterpreter, CommandQueue, Queueable};
use crate::renderable::Renderable;

pub enum Dist {
    By(Point),
    To(Point)
}

pub enum PawnCommand {
    Move(Dist),
    Idle,
    Die,
}

pub struct Character {
    pub renderable: Renderable,
    pub queueable: Queueable<PawnCommand>
}

impl CommandQueue for Character {
    type CommandType = PawnCommand;

    fn push(&mut self, command: Self::CommandType) {
        self.queueable.commands.push_back(command);
    }

    fn get_next(&mut self) -> Option<Self::CommandType> {
        self.queueable.commands.pop_front()
    }
}

impl CommandInterpreter for Character {
    fn interpret(&mut self, command: Self::CommandType) {
        match command {
            PawnCommand::Move(_) => {}
            PawnCommand::Idle => {}
            PawnCommand::Die => {}
        }
    }
}