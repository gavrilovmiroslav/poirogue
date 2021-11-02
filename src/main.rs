use bracket_lib::prelude::*;
use crate::game::map_builder::{SimpleDungeonBuilder};
use crate::game_state::{PoirogueGameManager};
use crate::map::{PoirogueMapBuilder};

mod map;
mod game_state;
mod entity;
mod game;

embedded_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");

fn main() -> BError {
    link_resource!(TILE_FONT, "../resources/classic_roguelike_white.png");

    let ctx = BTermBuilder::new()
        .with_tile_dimensions(16,16)
        .with_font("classic_roguelike_white.png", 8, 8)
        .with_simple_console(80, 50, "classic_roguelike_white.png")
        .with_title("Poirogue")
        .build()?;

    let mut manager: PoirogueGameManager =
        PoirogueGameManager::new(80, 50);

    let mut map_builder = SimpleDungeonBuilder::new();
    map_builder.generate(&mut manager.map);

    main_loop(ctx, manager)
}