use std::collections::VecDeque;
use bracket_lib::prelude::{Point, RGB};

pub enum Dist {
    By(Point),
    To(Point)
}

pub enum Command {
    Move(Dist),
    Idle,
    Die,
}

pub struct Character {
    pub glyph: char,
    pub position: Point,
    pub color: RGB,
    pub commands: VecDeque<Command>,
}
