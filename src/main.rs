use bracket_lib::prelude::{*};
use crate::game::{Game};
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

#[derive(Debug, StructOpt)]
#[structopt(name = "Poirogue")]
pub struct Opt {
    #[structopt(short = "seed", long, default_value = "0")]
    pub random_seed: u64,
    #[structopt(short = "r", long)]
    pub release_mode: bool,
    #[structopt(short = "b", long)]
    pub skip_binarize_on_boot: bool,
}

embedded_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
embedded_resource!(IMAGE_FONT, "../resources/MRMOTEXTEX_rexpaintx2.png");

fn main() {
    link_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
    link_resource!(IMAGE_FONT, "../resources/MRMOTEXTEX_rexpaintx2.png");

    Game::run(Opt::from_args());
}