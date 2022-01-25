use std::collections::HashSet;
use bracket_lib::prelude::Point;
use shipyard::{AddComponent, EntityId};
use crate::glyph::Glyph;

pub struct HasPosition(pub Point);
impl HasPosition {
    pub fn get_mut(&mut self) -> &mut Point {
        &mut self.0
    }
}

pub struct HasGlyph(pub Glyph);

pub struct HasSign(pub String);

pub struct HasSight {
    pub sight_distance: u8,
    pub field_of_view: HashSet<Point>,
}

pub struct IsPlayer;
pub struct PlayerPosition(pub Point);
pub struct Time(pub(crate) u64);
pub struct IsDirty(pub bool);

pub struct IsKnown;
pub struct IsCharacter;

pub struct IsDoor {
    pub sign: String,
    pub is_closed: bool,
    pub is_locked: Option<EntityId>,
}
