use bracket_lib::prelude::{*};
use crate::game::{Game};

mod rand_gen;
mod geometry;
mod game;
mod murder_gen;
mod map_gen;
mod map;
mod input;

fn main() {
    embedded_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");
    link_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");

    rand_gen::init_random_with_seed(0);
    
    Game::run();
}

/*
use bracket_noise::prelude::*;
use bracket_random::prelude::*;

use std::io::{stdout, Write};

struct NoiseState {
    pub rng: RandomNumberGenerator,
    pub noise: FastNoise,
}

impl NoiseState {
    pub fn new() -> NoiseState {
        let mut rng = RandomNumberGenerator::new();
        let mut noise = FastNoise::seeded(rng.next_u64());
        noise.set_noise_type(NoiseType::WhiteNoise);
        noise.set_frequency(0.5);
        noise.set_fractal_gain(0.5);
        noise.set_cellular_distance_function(CellularDistanceFunction::Manhattan);
        noise.set_cellular_return_type(CellularReturnType::CellValue);

        NoiseState { rng, noise }
    }
}

impl GameState for NoiseState {
    fn tick(&mut self, ctx: &mut BTerm) {
        for y in 0..50 {
            for x in 0..80 {
                let n = self.noise.get_noise(x as f32, y as f32);
                let col = (n + 1.0) * 0.5;
                ctx.print_color(x, y,RGB::from_f32(col, col, col), RGB::named(BLACK), "▒");
            }
        }
    }
}
fn main() {
    let (width, height) = (80, 50);
    let mut term = BTermBuilder::simple(width, height).unwrap().build().unwrap();

    let state = NoiseState::new();

    main_loop(term,state).unwrap();
}
*/