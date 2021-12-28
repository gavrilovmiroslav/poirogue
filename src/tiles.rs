use serde::{Serialize, Deserialize};
use strum_macros::Display;

pub type TileIndex = usize;
pub type RectIndex = usize;
pub type RoomIndex = usize;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Display)]
pub enum DoorState {
    Closed, Open,
}

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
    Door(DoorState),
    Wall
}

impl Default for MapTile {
    fn default() -> MapTile {
        MapTile::Obscured
    }
}

impl MapTile {
    pub fn is_blocking(&self) -> bool {
        match &self {
            MapTile::Debug(_) => true,
            MapTile::Obscured => true,
            MapTile::Wall => true,
            MapTile::Door(DoorState::Closed) => true,
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