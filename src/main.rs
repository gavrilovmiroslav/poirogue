mod game;

use bracket_lib::prelude::BTerm;
use game::Game;
use crate::game::GameSystem;

struct ExampleSystem;

impl GameSystem for ExampleSystem {
    fn init(&mut self) {
        println!("Example system init'd!");
    }

    fn tick(&mut self, ctx: &mut BTerm) {
        println!("Example system tick'd!");
    }
}

fn main() {
    let mut game = Game::new();
    let example_system = ExampleSystem{};
    game.add_system(example_system);
    game.run();
}
