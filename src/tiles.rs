use serde::{Serialize, Deserialize};
use strum_macros::Display;

pub type TileIndex = usize;
pub type RectIndex = usize;
pub type RoomIndex = usize;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Display)]
pub enum DebugMapTile {
    Construction(usize),
    RectCenter,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Display)]
pub enum MapTile {
    Debug(DebugMapTile),
    Obscured,
    Floor(usize),
    Stairs,
    Corridor,
    Door,
    Wall
}

impl Default for MapTile {
    fn default() -> MapTile {
        MapTile::Obscured
    }
}

impl MapTile {
    pub fn is_transparent(&self) -> bool {
        match &self {
            MapTile::Debug(_) => false,
            MapTile::Wall => false,
            MapTile::Obscured => false,
            _ => true
        }
    }
}