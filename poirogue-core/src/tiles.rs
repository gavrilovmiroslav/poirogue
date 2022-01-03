use bracket_color::prelude::RGB;
use serde::{Serialize, Deserialize};
use strum_macros::Display;
use crate::colors::Color;
use crate::glyph::{Glyph, GlyphOpt};
use crate::render_view::Colorable;

pub type TileIndex = usize;
pub type RectIndex = usize;
pub type RoomIndex = usize;

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Debug, Display, Hash)]
pub enum DoorState {
    Closed, Open,
}

impl DoorState {
    pub fn toggle(&self) -> DoorState {
        match self {
            DoorState::Closed => DoorState::Open,
            DoorState::Open => DoorState::Closed
        }
    }
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Debug, Display, Hash)]
pub enum DebugMapTile {
    Construction(usize),
    RectCenter,
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Debug, Display, Hash)]
pub enum MapTile {
    Debug(DebugMapTile),
    Obscured,
    Floor(usize),
    Stairs,
    Corridor,
    Door(DoorState),
}

#[repr(C, packed)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct MapTileRep {
    pub debug_construction: GlyphOpt,
    pub debug_rect_center: GlyphOpt,
    pub obscured: GlyphOpt,
    pub floor: GlyphOpt,
    pub stairs: GlyphOpt,
    pub corridor: GlyphOpt,
    pub door_opened: GlyphOpt,
    pub door_closed: GlyphOpt,
}

impl Default for MapTileRep {
    fn default() -> Self {
        MapTileRep {
            debug_construction: GlyphOpt::new(' '),
            debug_rect_center: GlyphOpt::new('*'),
            obscured: GlyphOpt::new('#'),
            floor: GlyphOpt::new('.'),
            stairs: GlyphOpt::new('>'),
            corridor: GlyphOpt::new('.'),
            door_opened: GlyphOpt::new('_'),
            door_closed: GlyphOpt::new('+'),
        }
    }
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

pub fn get_from_rep(s: &MapTile, p: &MapTileRep) -> GlyphOpt {
    use MapTile::*;
    use DebugMapTile::*;
    use DoorState::*;

    match s {
        Debug(Construction(_)) => p.debug_construction,
        Debug(RectCenter) => p.debug_rect_center,
        Obscured => p.obscured,
        Floor(_) => p.floor,
        Stairs => p.stairs,
        Corridor => p.corridor,
        Door(Closed) => p.door_closed,
        Door(Open) => p.door_opened,
    }
}