use bracket_color::prelude::RGB;
use serde::{Serialize, Deserialize};
use strum_macros::Display;
use crate::colors::Color;
use crate::glyph::{Glyph, GlyphOpt};

pub type TileIndex = usize;
pub type RectIndex = usize;
pub type RoomIndex = usize;

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Copy, Debug, Display, Hash)]
pub enum DebugMapTile {
    Construction(usize),
    RectCenter,
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Copy, Debug, Display, Hash)]
pub enum MapTile {
    Debug(DebugMapTile),
    Obscured,
    Floor(usize),
    Stairs,
    Corridor,
    Door,
}

impl Default for MapTile {
    fn default() -> MapTile {
        MapTile::Obscured
    }
}

impl MapTile {
    pub fn name(&self) -> String {
        (match &self {
            MapTile::Debug(_) => "Debug",
            MapTile::Obscured => "Obscured",
            MapTile::Floor(_) => "Floor",
            MapTile::Stairs => "Stairs",
            MapTile::Corridor => "Corridor",
            MapTile::Door => "Door",
        }).to_string()
    }

    pub fn is_blocking(&self) -> bool {
        match &self {
            MapTile::Debug(_) => true,
            MapTile::Obscured => true,
            MapTile::Door => true,
            _ => false
        }
    }

    pub fn is_obscured(&self) -> bool {
        match &self {
            MapTile::Obscured => true,
            _ => false,
        }
    }
}
