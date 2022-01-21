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
pub struct HasFieldOfView(pub Vec<Point>);
pub struct IsPlayer;
pub struct IsCharacter;
pub struct PlayerPosition(pub Point);
pub struct Time(pub(crate) u64);
pub struct IsDirty(pub bool);
pub struct HasSight(pub u8);
pub struct IsInvisible;
