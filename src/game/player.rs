use std::collections::VecDeque;
use bracket_lib::prelude::{Point, RGB, WHITE};
use crate::entity::PoirogueEntity;
use crate::game_state::Command;

pub struct Player {
    position: Point,
    commands: VecDeque<Command>,
}

impl Player {
    pub fn new(p: Point) -> Player {
        Player{ position: p, commands: VecDeque::new() }
    }
}

impl PoirogueEntity for Player {
    fn set_position(&mut self, p: Point) {
        self.position = p;
    }

    fn get_position(&self) -> Point { self.position }
    fn get_fg_color(&self) -> RGB { RGB::named(WHITE) }
    fn get_glyph(&self) -> char { '@' }

    fn get_next_command(&mut self) -> Option<Command> {
        self.commands.pop_front()
    }

    fn add_command(&mut self, comm: Command) {
        self.commands.push_back(comm);
    }

    fn clear_commands(&mut self) {
        self.commands.clear();
    }
}