use bracket_lib::prelude::{*};
use crate::game::{Game};
use crate::opt::Opt;
use structopt::*;

mod rex;
mod commands;
mod rand_gen;
mod game;
mod murder_gen;
mod map_gen;
mod map;
mod input;
mod render_view;
mod tiles;
mod readonly_archive_cave;
mod opt;
mod glyph;
mod entity;
mod colors;
mod core_systems;
mod json;
mod game_systems;
mod maybe;

const MAP_CONSOLE_LAYER: usize = 0;
const DRAWING_CONSOLE_LAYER: usize = 1;
const UI_CONSOLE_LAYER: usize = 2;

embedded_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
embedded_resource!(IMAGE_FONT, "../resources/MRMOTEXTEX_rexpaintx2.png");
embedded_resource!(TEXT_FONT, "../resources/8x8glyphs.png");

fn main() {
    link_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
    link_resource!(IMAGE_FONT, "../resources/MRMOTEXTEX_rexpaintx2.png");
    link_resource!(TEXT_FONT, "../resources/8x8glyphs.png");

    Game::run(Opt::from_args());
}