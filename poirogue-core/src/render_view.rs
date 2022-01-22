use std::collections::HashMap;
use std::ops::Mul;
use serde::{Serialize, Deserialize};
use serde::ser::{Serializer, SerializeMap};
use crate::rand_gen::get_random_between;
use crate::tiles::{DebugMapTile, get_from_rep, MapTile, MapTileRep};
use lru::{DefaultHasher, LruCache};
use lazy_static::*;
use std::sync::Mutex;
use bracket_color::prelude::{BLACK, DARK_CYAN, DARK_GRAY, GREEN, RED, WHITE};
use crate::colors::{Color, ColorShifter, named_color};

lazy_static! {
    static ref VIEW_REP_LRU: Mutex<LruCache<RenderView, MapTileRep>> = Mutex::new(LruCache::new(2));
}

pub trait Colorable {
    fn get_color(&self) -> Color;
}

pub trait RenderViewDefinition
{
    fn get_description(&self, t: &MapTile) -> String;
    fn get_glyph(&self, t: &MapTile) -> char;
    fn get_color(&self, t: &MapTile) -> Color;
    fn get_memory_color(&self, t: &MapTile) -> Color;
    fn get_see_all(&self) -> bool;
}

#[repr(u8)]
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Hash)]
pub enum RenderView {
    Game,
    Debug,
}

pub fn cache_render_view_rep(view: RenderView, rep: MapTileRep) {
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

impl RenderViewDefinition for RenderView {
    fn get_description(&self, t: &MapTile) -> String {
        match t {
            MapTile::Debug(DebugMapTile::Construction(_)) => "!Construction",
            MapTile::Debug(DebugMapTile::RectCenter) => "!Center",
            MapTile::Obscured => "Obscured",
            MapTile::Floor(_) => "Room",
            MapTile::Stairs => "Stairs",
            MapTile::Corridor => "Corridor",
            MapTile::Door => "Door",
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
                    MapTile::Obscured => '#',
                    MapTile::Floor(_) | MapTile::Corridor => '.',
                    MapTile::Door => '+',
                    MapTile::Stairs => '>',
                }
            }

            RenderView::Debug => {
                match t {
                    MapTile::Debug(Construction(n)) => (64 + *n as u8) as char,
                    MapTile::Debug(RectCenter) => '*',

                    MapTile::Obscured => ' ',
                    MapTile::Floor(n) => (64 + *n as u8) as char,
                    MapTile::Door => '+',
                    MapTile::Corridor => '.',
                    MapTile::Stairs => '>',
                }
            }
        }
    }

    fn get_color(&self, t: &MapTile) -> Color {
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
            MapTile::Debug(DebugMapTile::Construction(_)) => named_color(GREEN),
            MapTile::Debug(DebugMapTile::RectCenter) => named_color(RED),
            MapTile::Obscured => named_color(DARK_GRAY),
            MapTile::Floor(_) | MapTile::Corridor => named_color(DARK_CYAN),
            _ => named_color(WHITE),
        }
    }

    fn get_memory_color(&self, t: &MapTile) -> Color {
        named_color(DARK_GRAY).darken(0.5)
    }

    fn get_see_all(&self) -> bool {
        match self {
            RenderView::Game => false,
            RenderView::Debug => true,
        }
    }
}


