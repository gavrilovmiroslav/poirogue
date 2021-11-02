use std::collections::HashSet;
use bracket_lib::prelude::{GREEN, MAGENTA, RED, RGB, WHITE};
use rand::prelude::ThreadRng;
use rand::Rng;
use crate::map::{PoirogueTile};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum FloorTiles {
    Internal, Edge
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum TileKind {
    RectRoom(u8),
    Obscured(HashSet<u8>),
    Floor(u8, FloorTiles),
    Stairs,
    Corridor,
    Door,
    Center,
    Wall
}

impl TileKind {
    pub fn is_obscured(&self) -> bool {
        match &self {
            TileKind::Obscured(_) => true,
            _ => false
        }
    }

    pub fn is_wall(&self) -> bool {
        match &self {
            TileKind::Obscured(_) => true,
            TileKind::Wall => true,
            _ => false
        }
    }
}

impl Default for TileKind {
    fn default() -> TileKind {
        TileKind::Obscured(HashSet::new())
    }
}

impl PoirogueTile for TileKind {
    fn is_walkable(&self) -> bool {
        match &self {
            TileKind::Obscured(_) => false,
            TileKind::Wall | TileKind::Door => false,
            _ => true
        }
    }

    fn is_transparent(&self) -> bool {
        match &self {
            TileKind::RectRoom(_) => false,
            TileKind::Wall => false,
            TileKind::Obscured(_) => false,
            _ => true
        }
    }

    fn get_description(&self) -> String {
        return match &self {
            TileKind::RectRoom(_) => { "Floor".to_string() }
            TileKind::Floor(_, _) => { "Room".to_string() }
            TileKind::Obscured(_) => { "???".to_string() }
            TileKind::Stairs => { "Stairs".to_string() }
            TileKind::Corridor => { "Corridor".to_string() }
            TileKind::Door => { "Door".to_string() }
            TileKind::Center => { "Center".to_string() }
            TileKind::Wall => { "Wall".to_string() }
        }
    }

    fn get_glyph(&self) -> char {
        return match &self {
            TileKind::Obscured(_) => '?',
            TileKind::RectRoom(n) => (64 + n) as char,
            TileKind::Floor(_, _) | TileKind::Corridor => '.',
            TileKind::Door => '+',
            TileKind::Stairs => '>',
            TileKind::Center => '*',
            TileKind::Wall => '#'
        }
    }

    fn get_color(&self) -> RGB {
        let mut rng: ThreadRng = rand::thread_rng();

        match &self {
            TileKind::Obscured(_) => {
                let color = rng.gen_range(0.05..0.1);
                RGB::from_f32(color, color, color)
            },
            TileKind::Door => RGB::named(WHITE),
            TileKind::RectRoom(_) => RGB::named(GREEN),
            TileKind::Floor(_, _) | TileKind::Corridor | TileKind::Wall => {
                RGB::from_f32(
                    rng.gen_range(0.25..0.4),
                    rng.gen_range(0.25..0.4),
                    rng.gen_range(0.25..0.4))
            },
            TileKind::Stairs => RGB::named(MAGENTA),
            TileKind::Center => RGB::named(RED),
        }
    }
}