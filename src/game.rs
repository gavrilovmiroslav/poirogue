use bracket_lib::prelude::{BTerm, BTermBuilder, GameState, main_loop};
use std::sync::{Mutex, MutexGuard};
use crate::world::{get_world, World};
use lazy_static::lazy_static;

pub struct Game;

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        if let mut world = get_world() {
            for c in world.chars.iter() {
                ctx.print(1, 1, c.glyph);
            }
        }
    }
}

impl Game {
    pub fn run() {
        let game = Game {};
        let term = BTermBuilder::new()
            .with_tile_dimensions(16,16)
            .with_font("classic_roguelike_white.png", 8, 8)
            .with_simple_console(80, 50, "classic_roguelike_white.png")
            .with_title("Poirogue")
            .build().unwrap();

        main_loop(term, game).unwrap();
    }
}
