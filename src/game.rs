use std::borrow::BorrowMut;
use bracket_lib::prelude::*;
use bracket_lib::prelude::{BTerm, BTermBuilder, GameState, main_loop};
use crate::GameSystem;

pub struct Game {
    systems: Vec<Box<dyn GameSystem>>,
}

impl Game {
    pub fn new() -> Game {
        embedded_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
        link_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");

        Game { systems: vec![] }
    }

    pub fn add_system(&mut self, sys: impl GameSystem + 'static) {
        self.systems.push(Box::new(sys));
    }

    pub fn init(&mut self) {
        for sys in self.systems.iter_mut() {
            sys.init();
        }
    }

    pub fn run(mut self) {
        self.borrow_mut().init();

        let ctx = BTermBuilder::new()
            .with_tile_dimensions(16,16)
            .with_font("classic_roguelike_white.png", 8, 8)
            .with_simple_console(80, 50, "classic_roguelike_white.png")
            .with_title("Poirogue")
            .build().unwrap();

        main_loop(ctx, self).unwrap();
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        for sys in self.systems.iter_mut() {
            sys.tick(ctx);
        }
    }
}