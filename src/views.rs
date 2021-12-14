use bracket_lib::prelude::*;
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};
use crate::game::Entity;
use crate::rand_gen::get_random_between;
use crate::tiles::{DebugMapTile, MapTile};

pub trait View
{
    fn get_description(&self, t: &MapTile) -> String;
    fn get_glyph(&self, t: &MapTile) -> char;
    fn get_color(&self, t: &MapTile) -> RGB;
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum Views {
    Game,
    Debug,
}

impl Views {
    pub fn toggle(&self) -> Views {
        match &self {
            Views::Debug => Views::Game,
            Views::Game => Views::Debug
        }
    }
}

impl From<u8> for Views {
    fn from(n: u8) -> Self {
        match n {
            0 => Views::Game,
            _ => Views::Debug
        }
    }
}

impl From<Views> for u8 {
    fn from(n: Views) -> Self {
        match n {
            Views::Game => 0,
            Views::Debug => 1,
        }
    }
}

impl View for Views {
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

        match &self {
            Views::Game => {
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

            Views::Debug => {
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

