use std::collections::HashMap;
use bracket_lib::prelude::*;
use serde::{Serialize, Deserialize};
use serde::ser::{Serializer, SerializeMap};
use crate::rand_gen::get_random_between;
use crate::tiles::{DebugMapTile, DoorState, get_from_rep, MapTile, MapTileRep};
use lru::{DefaultHasher, LruCache};
use lazy_static::*;
use std::sync::Mutex;
use crate::game::GameSharedData;

lazy_static! {
    static ref VIEW_REP_LRU: Mutex<LruCache<RenderView, MapTileRep>> = Mutex::new(LruCache::new(2));
}

pub trait View<Tile>
{
    fn get_description(&self, t: &Tile) -> String;
    fn get_glyph(&self, t: &Tile) -> char;
    fn get_color(&self, t: &Tile) -> RGB;
    fn get_see_all(&self) -> bool;
}

#[repr(u8)]
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Hash)]
pub enum RenderView {
    Game,
    Debug,
}

pub fn add_render_view_rep(view: RenderView, rep: MapTileRep) {
    let mut cache = VIEW_REP_LRU.lock().unwrap();
    cache.put(view, rep);
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

impl View<MapTile> for RenderView {
    fn get_description(&self, t: &MapTile) -> String {
        match t {
            MapTile::Debug(DebugMapTile::Construction(_)) => "!Construction",
            MapTile::Debug(DebugMapTile::RectCenter) => "!Center",
            MapTile::Obscured => "Obscured",
            MapTile::Floor(_) => "Room",
            MapTile::Stairs => "Stairs",
            MapTile::Corridor => "Corridor",
            MapTile::Door(_) => "Door",
            MapTile::Wall => "Wall",
        }.to_string()
    }

    fn get_glyph(&self, t: &MapTile) -> char {
        use DebugMapTile::*;

        {
            let mut cache = VIEW_REP_LRU.lock().unwrap();
            if let Some(foo) = cache.get(self) {
                let glyph = get_from_rep(t, foo);
                if glyph.ch.is_some() {
                    return glyph.ch.unwrap();
                }
            }
        }

        match &self {
            RenderView::Game => {
                match t {
                    MapTile::Debug(_) => '!',
                    MapTile::Obscured | MapTile::Wall => '#',
                    MapTile::Floor(_) | MapTile::Corridor => '.',
                    MapTile::Door(DoorState::Closed) => '+',
                    MapTile::Door(DoorState::Open) => '-',
                    MapTile::Stairs => '>',
                }
            }

            RenderView::Debug => {
                match t {
                    MapTile::Debug(Construction(n)) => (64 + *n as u8) as char,
                    MapTile::Debug(RectCenter) => '*',

                    MapTile::Obscured => ' ',
                    MapTile::Floor(n) => (64 + *n as u8) as char,
                    MapTile::Door(DoorState::Closed) => '+',
                    MapTile::Door(DoorState::Open) => '-',
                    MapTile::Corridor => '.',
                    MapTile::Stairs => '>',
                    MapTile::Wall => '#'
                }
            }
        }
    }

    fn get_color(&self, t: &MapTile) -> RGB {
        {
            let mut cache = VIEW_REP_LRU.lock().unwrap();
            if let Some(foo) = cache.get(self) {
                let glyph = get_from_rep(t, foo);
                if glyph.fg.is_some() {
                    return glyph.fg.unwrap();
                }
            }
        }

        match t {
            MapTile::Debug(DebugMapTile::Construction(_)) => RGB::named(GREEN),
            MapTile::Debug(DebugMapTile::RectCenter) => RGB::named(RED),
            MapTile::Obscured => RGB::named(DARK_GRAY),
            MapTile::Floor(_) | MapTile::Corridor => {
                let get_value = || { get_random_between(0.25, 0.55) };
                RGB::from_f32(get_value(), get_value(), get_value())
            },
            _ => RGB::named(WHITE),
        }
    }

    fn get_see_all(&self) -> bool {
        match self {
            RenderView::Game => false,
            RenderView::Debug => true,
        }
    }
}

