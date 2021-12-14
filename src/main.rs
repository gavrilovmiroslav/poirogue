use bracket_lib::prelude::{*};
use crate::game::{Game};
use crate::opt::Opt;
use structopt::*;

mod rex;
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
mod readonly_archive_cave;
mod opt;

embedded_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
embedded_resource!(IMAGE_FONT, "../resources/MRMOTEXTEX_rexpaintx2.png");
embedded_resource!(TEXT_FONT, "../resources/8x8glyphs.png");

fn main() {
    link_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
    link_resource!(IMAGE_FONT, "../resources/MRMOTEXTEX_rexpaintx2.png");
    link_resource!(TEXT_FONT, "../resources/8x8glyphs.png");

    Game::run(Opt::from_args());
}