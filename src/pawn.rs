use std::collections::VecDeque;
use bracket_lib::prelude::{Point, RGB};
use crate::command_queue::{Queueable};
use crate::geometry::Dist;
use crate::drawable::Drawable;

pub enum PawnCommand {
    Move(Dist),
    Idle,
    Die,
}

pub struct Pawn {
    pub drawable: Drawable,
    pub commands: Queueable<PawnCommand>
}
