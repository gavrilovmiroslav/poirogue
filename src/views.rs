use bracket_lib::prelude::*;
use crate::game::Entity;
use crate::rand_gen::get_random_between;
use crate::tiles::{DebugMapTile, MapTile};

pub trait View
{
    fn get_description(&self, t: &MapTile) -> String;
    fn get_glyph(&self, t: &MapTile) -> char;
    fn get_color(&self, t: &MapTile) -> RGB;
}

pub struct DebugView;

impl View for DebugView {
    fn get_description(&self, t: &MapTile) -> String {
        match t {
            MapTile::Debug(DebugMapTile::Construction(_)) => "!Construction",
            MapTile::Debug(DebugMapTile::RectCenter) => "!Center",

            MapTile::Obscured => "???",
            MapTile::Floor(_) => "Room",
            MapTile::Stairs => "Stairs",
            MapTile::Corridor => "Corridor",
            MapTile::Door => "Door",
            MapTile::Wall => "Wall",
        }.to_string()
    }

    fn get_glyph(&self, t: &MapTile) -> char {
        use DebugMapTile::*;

        match t {
            MapTile::Debug(Construction(n)) => (64 + *n as u8) as char,
            MapTile::Debug(RectCenter) => '*',

            MapTile::Obscured => ' ',
            MapTile::Floor(n) => (64 + *n as u8) as char,
            MapTile::Corridor => '-',
            MapTile::Door => '+',
            MapTile::Stairs => '>',
            MapTile::Wall => '#'
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

pub struct GameView;
impl View for GameView {
    fn get_description(&self, t: &MapTile) -> String {
        match t {
            MapTile::Debug(_) => panic!("THIS SHOULD NOT HAPPEN!"),

            MapTile::Obscured => "???",
            MapTile::Floor(_) => "Room",
            MapTile::Stairs => "Stairs",
            MapTile::Corridor => "Corridor",
            MapTile::Door => "Door",
            MapTile::Wall => "Wall",
        }.to_string()
    }

    fn get_glyph(&self, t: &MapTile) -> char {
        use DebugMapTile::*;

        match t {
            MapTile::Debug(_) => '!',
            MapTile::Obscured => '#',
            MapTile::Floor(n) => '.',
            MapTile::Corridor => '.',
            MapTile::Door => '+',
            MapTile::Stairs => '>',
            MapTile::Wall => '#'
        }
    }

    fn get_color(&self, t: &MapTile) -> RGB {
        match t {
            MapTile::Debug(_) => RGB::named(RED),
            MapTile::Obscured => {
                let color = get_random_between(0.05, 0.1);
                RGB::from_f32(color, color, color)
            },
            _ => RGB::named(WHITE),
        }
    }
}
