use std::borrow::{Borrow, BorrowMut};
use serde::{Serialize, Deserialize};
use std::collections::{HashSet, VecDeque};
use bracket_lib::prelude::*;
use object_pool::Reusable;
use crate::commands::GameCommand;
use crate::geometry::Glyph;
use crate::rand_gen::get_random_between;
use crate::tiles::{MapTile, TileIndex};
use crate::render_view::{View};

#[derive(Default)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<MapTile>,
    pub visible: Vec<bool>, // !is_transparent
    pub blocked: Vec<bool>, // !is_walkable
    pub revealed: Vec<bool>,
}

impl Map {

    // HELPER FUNCTIONS

    fn is_valid_tile(&self, x:i32, y:i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    pub fn is_tile(&self, tile: TileIndex, tile_kind: MapTile) -> bool {
        let p = self.index_to_point2d(tile);
        if self.is_valid_tile(p.x, p.y) {
            self.tiles[tile] == tile_kind
        } else { false }
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

    // MAIN FUNCTIONS

    pub fn new(w: i32, h: i32) -> Self {
        let mut tiles = Vec::new();
        let mut visible = Vec::new();
        let mut revealed = Vec::new();
        let mut blocked = Vec::new();

        for _i in 0 .. (w * h) {
            tiles.push(MapTile::default());
            visible.push(true);
            revealed.push(true);
            blocked.push(true);
        }

        Map { width: w, height: h, tiles, visible, revealed, blocked }
    }

    pub fn is_tile_transparent_xy(&self, x: i32, y: i32) -> bool {
        match self.get_tile_index(x, y) {
            Some(index) => self.tiles[index].is_transparent(),
            None => false
        }
    }

    pub fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        let index = self.get_tile_index(x, y).unwrap();

        !self.blocked[index] && self.revealed[index]
    }

    pub fn set(&mut self, x: i32, y: i32, t: MapTile) {
        match self.get_tile_index(x, y) {
            Some(index) => {
                self.tiles[index] = t.clone();
                self.blocked[index] = false; // !t.is_walkable();
                self.visible[index] = true; // !t.is_transparent();
            },
            None => ()
        }
    }
}

impl Map {
    pub fn render(&self, ctx: &mut BTerm, view: &dyn View) {
        let mut index: usize = 0;

        let mut batch = DrawBatch::new();

        for y in 0 .. self.height {
            for x in 0 .. self.width {
                let tile = &self.tiles[index];

                if self.revealed[index] {
                    let color = if !self.visible[index] { view.get_color(tile) } else { RGB::named(GREY) };
                    batch.print_color(Point::new(x, y), view.get_glyph(tile), ColorPair::new(color, RGB::named(BLACK)));
                }
                index += 1;
            }
        }

        batch.submit(0).unwrap();
        render_draw_buffer(ctx).unwrap();
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, index: usize) -> bool {
        let (x, y) = self.get_tile_coords(index);

        if !self.is_tile_transparent_xy(x, y) { return true; }

        return false;
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
        self.get_tile_index(pt.x, pt.y).unwrap()
    }

    fn index_to_point2d(&self, index: usize) -> Point {
        self.get_tile_point(index)
    }

    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}
