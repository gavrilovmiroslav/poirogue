use serde::{Serialize, Deserialize};
use std::collections::{HashSet, VecDeque};
use std::collections::hash_set::Iter;
use std::iter::Filter;
use std::ops::Mul;
use bracket_lib::prelude::*;
use object_pool::Reusable;
use crate::commands::GameCommand;
use crate::tiles::{MapTile, TileIndex};
use crate::render_view::{RenderViewDefinition};
use lru::{LruCache};
use lazy_static::*;
use std::sync::Mutex;
use crate::colors::{Color, named_color, ColorShifter};
use crate::entity::PlayerPosition;
use crate::game::Store;
use crate::rand_gen::get_random_between;

lazy_static! {
    static ref VISIBLE_MEMORY: Mutex<LruCache<TileIndex, u8>> = Mutex::new(LruCache::new(64));
}

#[derive(Default)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<MapTile>,
}

impl Map {
    fn is_valid_tile(&self, x:i32, y:i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    pub fn is_tile(&self, tile: TileIndex, tile_kind: MapTile) -> bool {
        let p = self.index_to_point2d(tile);
        if self.is_valid_tile(p.x, p.y) {
            self.tiles[tile] == tile_kind
        } else { false }
    }

    pub fn get_tile_at(&self, tile_index: TileIndex) -> &MapTile {
        &self.tiles[tile_index]
    }

    pub fn get_tile_index_from_point(&self, p: Point) -> Option<usize> {
        if self.is_valid_tile(p.x, p.y) {
            Some(((p.y * self.width) + p.x) as usize)
        } else {
            None
        }
    }

    pub fn get_tile_index(&self, x: i32, y: i32) -> Option<usize> {
        if self.is_valid_tile(x, y) {
            Some(((y * self.width) + x) as usize)
        } else {
            None
        }
    }

    pub fn get_tile_coords(&self, index: usize) -> (i32, i32) {
        (index as i32 % self.width, index as i32 / self.width)
    }

    fn get_tile_point(&self, index: usize) -> Point {
        Point{ x: index as i32 % self.width, y: index as i32 / self.width }
    }


    pub fn new(w: i32, h: i32) -> Self {
        let size = (w * h) as usize;
        let mut tiles = Vec::with_capacity(size);

        for _ in 0..size {
            tiles.push(MapTile::default());
        }

        Map { width: w, height: h, tiles, }
    }

    pub fn is_tile_blocked(&self, tile_index: TileIndex) -> bool {
        self.tiles[tile_index].is_blocking()
    }

    pub fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        let index = self.get_tile_index(x, y).unwrap();
        !self.is_tile_blocked(index)
    }

    pub fn set_at_tile_index(&mut self, tile_index: TileIndex, t: MapTile) {
        self.tiles[tile_index] = t.clone();
    }

    pub fn set(&mut self, x: i32, y: i32, t: MapTile) {
        if let Some(index) = self.get_tile_index(x, y) {
            self.tiles[index] = t.clone();
        }
    }

    pub fn get_all_doors(&self) -> Vec<TileIndex> {
        self.tiles.iter().enumerate()
            .filter(|(_, &t)| t == MapTile::Door)
            .map(|(i, _)| i)
            .collect::<Vec<TileIndex>>()
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, index: usize) -> bool {
        self.is_tile_blocked(index)
    }

    fn get_available_exits(&self, index: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits : SmallVec<[(usize, f32); 10]> = SmallVec::new();
        let (x, y) = self.get_tile_coords(index);
        let w = self.width as usize;

        // Cardinal directions
        exits.push(if self.is_exit_valid(x - 1, y) { (index - 1, 1.0) } else { (0, 0.0) });
        exits.push(if self.is_exit_valid(x + 1, y) { (index + 1, 1.0) } else { (0, 0.0) });
        exits.push(if self.is_exit_valid(x, y - 1) { (index - w, 1.0) } else { (0, 0.0) });
        exits.push(if self.is_exit_valid(x, y + 1) { (index + w, 1.0) } else { (0, 0.0) });

        // Diagonals
        exits.push(if self.is_exit_valid(x - 1, y - 1) { ((index - w) - 1, 1.4) } else { (0, 0.0) });
        exits.push(if self.is_exit_valid(x + 1, y - 1) { ((index - w) + 1, 1.4) } else { (0, 0.0) });
        exits.push(if self.is_exit_valid(x - 1, y + 1) { ((index + w) - 1, 1.4) } else { (0, 0.0) });
        exits.push(if self.is_exit_valid(x + 1, y + 1) { ((index + w) + 1, 1.4) } else { (0, 0.0) });

        exits
    }

    fn get_pathing_distance(&self, index1: usize, index2: usize) -> f32 {
        let p1 = self.get_tile_point(index1);
        let p2 = self.get_tile_point(index2);
        DistanceAlg::Manhattan.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn point2d_to_index(&self, pt : Point) -> usize {
        self.get_tile_index(pt.x, pt.y).unwrap_or(0)
    }

    fn index_to_point2d(&self, index: usize) -> Point {
        self.get_tile_point(index)
    }

    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}
