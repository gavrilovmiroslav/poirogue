use std::borrow::{Borrow, BorrowMut};
use serde::{Serialize, Deserialize};
use std::collections::{HashSet, VecDeque};
use specs::{Component, VecStorage};
use bracket_lib::prelude::*;
use object_pool::Reusable;
use crate::game::{GameCommand, OrderedDrawBatch, RenderBrain};
use crate::geometry::Glyph;
use crate::rand_gen::get_random_between;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum FloorTiles {
    Internal, Edge
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum MapTile {
    Construction(usize),
    Obscured,
    Floor(usize),
    Stairs,
    Corridor,
    Door,
    Center,
    Wall
}

impl Default for MapTile {
    fn default() -> MapTile {
        MapTile::Obscured
    }
}

impl MapTile {
    pub fn is_obscured(&self) -> bool {
        match &self {
            MapTile::Obscured => true,
            _ => false
        }
    }

    pub fn is_wall(&self) -> bool {
        match &self {
            MapTile::Obscured => true,
            MapTile::Wall => true,
            _ => false
        }
    }

    fn is_walkable(&self) -> bool {
        match &self {
            MapTile::Obscured => false,
            MapTile::Wall | MapTile::Door => false,
            _ => true
        }
    }

    fn is_transparent(&self) -> bool {
        match &self {
            MapTile::Construction(_) => false,
            MapTile::Wall => false,
            MapTile::Obscured => false,
            _ => true
        }
    }

    fn get_description(&self) -> String {
        return match &self {
            MapTile::Construction(_) => { "Floor".to_string() }
            MapTile::Floor(_) => { "Room".to_string() }
            MapTile::Obscured => { "???".to_string() }
            MapTile::Stairs => { "Stairs".to_string() }
            MapTile::Corridor => { "Corridor".to_string() }
            MapTile::Door => { "Door".to_string() }
            MapTile::Center => { "Center".to_string() }
            MapTile::Wall => { "Wall".to_string() }
        }
    }

    fn get_glyph(&self) -> char {
        return match &self {
            MapTile::Obscured => '#',
            MapTile::Construction(n) => (64 + *n as u8) as char,
            MapTile::Floor(n) => '.', //(64 + n) as char,
            MapTile::Corridor => '.',
            MapTile::Door => '+',
            MapTile::Stairs => '>',
            MapTile::Center => '*',
            MapTile::Wall => '#'
        }
    }

    fn get_color(&self) -> RGB {
        match &self {
            MapTile::Obscured => {
                let color = get_random_between(0.05, 0.1);
                RGB::from_f32(color, color, color)
            },
            MapTile::Door => RGB::named(WHITE),
            MapTile::Construction(_) => RGB::named(GREEN),
            MapTile::Floor(_) | MapTile::Corridor | MapTile::Wall => {
                RGB::from_f32(
                    get_random_between(0.25, 0.4),
                    get_random_between(0.25, 0.4),
                    get_random_between(0.25, 0.4))
            },
            MapTile::Stairs => RGB::named(MAGENTA),
            MapTile::Center => RGB::named(RED),
        }
    }
}

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

    #[inline(always)]
    fn is_valid_tile(&self, x:i32, y:i32) -> bool {
        x >= 0 && x <= self.width && y >= 0 && y <= self.height
    }

    #[inline(always)]
    pub fn get_tile_index(&self, x: i32, y: i32) -> Option<usize> {
        if self.is_valid_tile(x, y) {
            Some(((y * self.width) + x) as usize)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_tile_coords(&self, index: usize) -> (i32, i32) {
        (index as i32 % self.width, index as i32 / self.width)
    }

    #[inline(always)]
    fn get_tile_point(&self, index: usize) -> Point {
        Point{ x: index as i32 % self.width, y: index as i32 / self.width }
    }

    pub fn get_neighbors(&self, x: i32, y: i32) -> [MapTile; 4] {
        let mut result: [MapTile; 4 ] = Default::default();
        result[0] = self.tiles[self.get_tile_index(x, y - 1).unwrap()].clone();
        result[1] = self.tiles[self.get_tile_index(x + 1, y).unwrap()].clone();
        result[2] = self.tiles[self.get_tile_index(x, y + 1).unwrap()].clone();
        result[3] = self.tiles[self.get_tile_index(x - 1, y).unwrap()].clone();
        return result;
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

    pub fn is_tile_walkable(&self, pos: Point) -> bool {
        match self.get_tile_index(pos.x, pos.y) {
            Some(index) => self.tiles[index].is_walkable(),
            None => false
        }
    }

    pub fn is_tile_revealed(&self, pos: Point) -> bool {
        match self.get_tile_index(pos.x, pos.y) {
            _ => true,
/*            Some(index) => self.revealed[index],
            None => false*/
        }
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

impl RenderBrain for Map {
    fn render(&self) -> OrderedDrawBatch {
        // #[inline(always)]
        fn draw_tile(this: &Map, batch: &mut DrawBatch, index: usize, x: i32, y: i32) {
            let tile = &this.tiles[index];

            if this.revealed[index] {
                let color = if !this.visible[index] { tile.get_color() } else { RGB::named(GREY) };
                batch.print_color(Point::new(x, y), tile.get_glyph(), ColorPair::new(color, RGB::named(BLACK)));
            }
        }

        let depth = 0;
        let mut draw_batch = DrawBatch::new();
        draw_batch.target(0);
        let mut index: usize = 0;

        for y in 0 .. self.height {
            for x in 0 .. self.width {
                draw_tile(self, draw_batch.borrow_mut(), index, x, y);
                index += 1;
            }
        }

        OrderedDrawBatch::new(depth, draw_batch)
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
        if self.is_exit_valid(x - 1, y) { exits.push((index - 1, 1.0)) };
        if self.is_exit_valid(x + 1, y) { exits.push((index + 1, 1.0)) };
        if self.is_exit_valid(x, y - 1) { exits.push((index - w, 1.0)) };
        if self.is_exit_valid(x, y + 1) { exits.push((index + w, 1.0)) };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) { exits.push(((index - w) - 1, 1.4)); }
        if self.is_exit_valid(x + 1, y - 1) { exits.push(((index - w) + 1, 1.4)); }
        if self.is_exit_valid(x - 1, y + 1) { exits.push(((index + w) - 1, 1.4)); }
        if self.is_exit_valid(x + 1, y + 1) { exits.push(((index + w) + 1, 1.4)); }

        exits
    }

    fn get_pathing_distance(&self, index1: usize, index2: usize) -> f32 {
        let p1 = self.get_tile_point(index1);
        let p2 = self.get_tile_point(index2);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
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