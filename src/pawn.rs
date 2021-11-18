use std::collections::VecDeque;
use bracket_lib::prelude::{BTerm, Point, RGB, VirtualKeyCode, WHITE};
use crate::command_queue::{Queueable};
use crate::geometry::Dist;
use crate::drawable::Drawable;

pub enum PawnRole {
    Player,
    Mob
}

pub enum PawnCommand {
    Move(Dist),
    Idle,
    Die,
}

pub struct Pawn {
    pub role: PawnRole,
    pub drawable: Drawable,
    pub commands: Queueable<PawnCommand>
}

impl Pawn {
    pub fn new_player(at: Point) -> Pawn {
        Pawn { role: PawnRole::Player, drawable: Drawable::new('@', at, RGB::named(WHITE)), commands: Queueable::default() }
    }

    fn update_player(&mut self, ctx: &mut BTerm) {
        while let Some(comm) = self.commands.get_next() {
            match comm {
                PawnCommand::Move(Dist::By(pt)) => {
                    self.drawable.position += pt;
                },
                _ => {}
            }
        }

        match ctx.key {
            Some(VirtualKeyCode::Up) => self.commands.push(PawnCommand::Move(Dist::By(Point::new(0, -1)))),
            Some(VirtualKeyCode::Down) => self.commands.push(PawnCommand::Move(Dist::By(Point::new(0, 1)))),
            Some(VirtualKeyCode::Left) => self.commands.push(PawnCommand::Move(Dist::By(Point::new(-1, 0)))),
            Some(VirtualKeyCode::Right) => self.commands.push(PawnCommand::Move(Dist::By(Point::new(1, 0)))),
            _ => {}
        }
    }

    pub fn update(&mut self, ctx: &mut BTerm) {
        match self.role {
            PawnRole::Player => self.update_player(ctx),
            PawnRole::Mob => {}
        }
    }
}
