use bracket_lib::prelude::{Point, RGB};
use crate::game_state::Command;

pub trait PoirogueEntity {
    fn set_position(&mut self, p: Point);
    fn get_position(&self) -> Point;
    fn get_fg_color(&self) -> RGB;
    fn get_glyph(&self) -> char;
    fn get_next_command(&mut self) -> Option<Command>;
    fn add_command(&mut self, comm: Command);
    fn clear_commands(&mut self);
}