use bracket_lib::prelude::{BTerm, BTermBuilder, GameState, main_loop, Point, VirtualKeyCode};
use std::sync::{Mutex, MutexGuard};
use crate::world::{get_world, Tick, World};
use lazy_static::lazy_static;
use crate::console::Console;
use crate::geometry::Dist;
use crate::pawn::PawnCommand;

pub struct Game {
    console: Console,
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        if let mut world= get_world() {
            world.map.update_player_fov();
            world.map.render(ctx);
            world.receive_commands();
            world.update(ctx);
        }

        self.console.tick(ctx);
    }
}

impl Game {
    pub fn run() {
        let game = Game { console: Console::new() };

        let term =
            BTermBuilder::new()
            .with_tile_dimensions(16,16)
            .with_font("classic_roguelike_white.png", 8, 8)
            .with_simple_console(80, 50, "classic_roguelike_white.png")
            .with_title("Poirogue")
            .build().unwrap();

        main_loop(term, game).unwrap();
    }
}
