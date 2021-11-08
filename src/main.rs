mod game;
mod system;

use bracket_lib::prelude::BTerm;
use game::Game;
use crate::game::GameSystem;
use crate::system::GameSystem;

struct ExampleSystem;

impl GameSystem for ExampleSystem {
    fn get_name(&self) -> &str { "Example system" }
    fn init(&mut self) {}
    fn tick(&mut self, _ctx: &mut BTerm) {}
}

fn main() {
    let mut game = Game::new();
    let example_system = ExampleSystem{};
    game.add_system(example_system);
    game.run();
}
