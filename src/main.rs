use bracket_lib::prelude::{*};
use crate::game::{Game};

mod geometry;
mod drawable;
mod command_queue;
mod world;
mod game;
mod pawn;
mod console;
mod murder_gen;
mod floor_builder;
mod map;

fn main() {
    embedded_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
    link_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");

    Game::run();
}
