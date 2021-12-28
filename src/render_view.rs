use bracket_lib::prelude::*;
use serde::{Serialize, Deserialize};
use crate::rand_gen::get_random_between;
use crate::tiles::{DebugMapTile, DoorState, MapTile};

pub trait View
{
    fn get_description(&self, t: &MapTile) -> String;
    fn get_glyph(&self, t: &MapTile) -> char;
    fn get_color(&self, t: &MapTile) -> RGB;
}

#[repr(u8)]
#[derive(PartialEq, Serialize, Deserialize)]
pub enum RenderView {
    Game,
    Debug,
}

impl RenderView {
    pub fn toggle(&self) -> RenderView {
        match &self {
            RenderView::Debug => RenderView::Game,
            RenderView::Game => RenderView::Debug
        }
    }
}

impl From<u8> for RenderView {
    fn from(n: u8) -> Self {
        match n {
            0 => RenderView::Game,
            _ => RenderView::Debug
        }
    }
}

impl From<RenderView> for u8 {
    fn from(n: RenderView) -> Self {
        match n {
            RenderView::Game => 0,
            RenderView::Debug => 1,
        }
    }
}

impl View for RenderView {
    fn get_description(&self, t: &MapTile) -> String {
        match t {
            MapTile::Debug(DebugMapTile::Construction(_)) => "!Construction",
            MapTile::Debug(DebugMapTile::RectCenter) => "!Center",

            MapTile::Obscured => "???",
            MapTile::Floor(_) => "Room",
            MapTile::Stairs => "Stairs",
            MapTile::Corridor => "Corridor",
            MapTile::Door(_) => "Door",
            MapTile::Wall => "Wall",
        }.to_string()
    }

    fn get_glyph(&self, t: &MapTile) -> char {
        use DebugMapTile::*;

        match &self {
            RenderView::Game => {
                match t {
                    MapTile::Debug(_) => '!',
                    MapTile::Obscured => '#',
                    MapTile::Floor(n) => '.',
                    MapTile::Corridor => '.',
                    MapTile::Door(DoorState::Closed) => '+',
                    MapTile::Door(DoorState::Open) => '-',
                    MapTile::Stairs => '>',
                    MapTile::Wall => '#'
                }
            }

            RenderView::Debug => {
                match t {
                    MapTile::Debug(Construction(n)) => (64 + *n as u8) as char,
                    MapTile::Debug(RectCenter) => '*',

                    MapTile::Obscured => ' ',
                    MapTile::Floor(n) => (64 + *n as u8) as char,
                    MapTile::Corridor => '-',
                    MapTile::Door(DoorState::Closed) => '+',
                    MapTile::Door(DoorState::Open) => '-',
                    MapTile::Stairs => '>',
                    MapTile::Wall => '#'
                }
            }
        }
    }

    fn get_color(&self, t: &MapTile) -> RGB {
        match t {
            MapTile::Debug(DebugMapTile::Construction(_)) => RGB::named(GREEN),
            MapTile::Debug(DebugMapTile::RectCenter) => RGB::named(RED),
            MapTile::Obscured => {
                let color = get_random_between(0.05, 0.1);
                RGB::from_f32(color, color, color)
            },
            _ => RGB::named(WHITE),
        }
    }
}

