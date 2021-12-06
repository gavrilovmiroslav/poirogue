use bracket_lib::prelude::{*};
use crate::game::{Game};

mod commands;
mod render;
mod rand_gen;
mod geometry;
mod game;
mod murder_gen;
mod map_gen;
mod map;
mod input;
mod views;
mod tiles;
mod views_impl;

fn main() {
    embedded_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
    link_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");

    rand_gen::init_random_with_seed(0);
    
    Game::run();
}