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

pub struct HasFieldOfView {
    pub fov: HashSet<Point>,
    pub distance: u8,
    pub vision: String,
}

impl HasFieldOfView {
    pub fn new(d: u8, v: &str) -> HasFieldOfView {
        HasFieldOfView{
            fov: HashSet::new(),
            distance: d,
            vision: v.to_string()
        }
    }
}

pub struct HasGlyph(pub Glyph);
pub struct IsPlayer;
pub struct IsCharacter;
pub struct PlayerPosition(pub Point);
pub struct Time(pub(crate) u64);
pub struct IsDirty(pub bool);
pub struct IsInvisible;
