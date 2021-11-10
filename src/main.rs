use bracket_lib::prelude::{*};
use crate::game::{Game};

mod renderable;
mod command_queue;
mod world;
mod game;
mod characters;

fn main() {
    embedded_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
    link_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");

    Game::run();
}
