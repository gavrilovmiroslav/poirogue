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

pub struct Player {
    pub entity: Option<EntityId>,
    pub cached_position: Point,
}

#[derive(Default)]
pub struct HasCooldown {
    pub duration: u16,
    pub skip_after: Option<u16>,
}

impl HasCooldown {
    pub fn with_skip(t: u16) -> HasCooldown {
        HasCooldown{ duration: 0, skip_after: Some(t) }
    }
}

pub struct IsDirty(pub bool);

pub struct IsKnown;
pub struct IsCharacter;

pub struct IsDoor {
    pub sign: String,
    pub is_closed: bool,
    pub is_locked: Option<EntityId>,
}
