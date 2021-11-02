use bracket_lib::prelude::{HSV, Point, RGB};
use rand::{Rng, thread_rng};
use crate::entity::PoirogueEntity;
use crate::game_state::Command;

pub struct Mob {
    pub position: Point,
    pub symbol: char,
    pub color: RGB,
}

impl Mob {
    pub fn new(p: Point) -> Mob {
        let mut rand = thread_rng();
        let color = HSV::from_f32(
            rand.gen_range(0.0 .. 1.0),
            rand.gen_range(0.0 .. 1.0),
            rand.gen_range(0.7 .. 1.0)).to_rgb();

        Mob{
            position: p,
            symbol: rand.gen_range('A'..'z'),
            color
        }
    }
}

impl PoirogueEntity for Mob {
    fn set_position(&mut self, p: Point) { self.position = p; }
    fn get_position(&self) -> Point { self.position }
    fn get_fg_color(&self) -> RGB {  self.color }
    fn get_glyph(&self) -> char { self.symbol }
    fn get_next_command(&mut self) -> Option<Command> { None }
    fn add_command(&mut self, _comm: Command) {}
    fn clear_commands(&mut self) {}
}