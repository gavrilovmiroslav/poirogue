use std::collections::VecDeque;

pub trait CommandQueue {
    type CommandType: Sync + Send;

    fn push(&mut self, command: Self::CommandType);
    fn get_next(&mut self) -> Option<Self::CommandType>;
}

pub trait CommandInterpreter : CommandQueue {
    fn interpret(&mut self, command: Self::CommandType);
}

pub struct Queueable<Commands> {
    pub commands: VecDeque<Commands>,
}