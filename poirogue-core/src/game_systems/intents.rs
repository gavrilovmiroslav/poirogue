use bracket_lib::prelude::Point;
use crate::tiles::TileIndex;
use shipyard::EntityId;

pub struct BumpIntent {
    pub bumper: EntityId,
    pub pos: Point,
}

pub struct UnlockIntent {
    pub entity: EntityId,
    pub target: EntityId,
}

pub struct CollectIntent {
    pub collector: EntityId,
    pub item: EntityId,
}
