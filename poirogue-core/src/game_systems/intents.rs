use crate::tiles::TileIndex;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BumpIntent {
    pub entity: u64,
    pub pos: (i32, i32),
}

#[derive(Serialize, Deserialize)]
pub struct UnlockIntent {
    pub entity: u64,
    pub tile: TileIndex,
}

