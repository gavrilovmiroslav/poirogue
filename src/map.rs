use bracket_lib::prelude::{Algorithm2D, BaseMap, BLACK, BTerm, DistanceAlg, field_of_view_set, GREY, Point, RGB, SmallVec};
use crate::entity::PoirogueEntity;
use crate::game::tiles::TileKind;

pub trait PoirogueTile : Default + Clone + 'static + Sized {
    fn is_walkable(&self) -> bool;
    fn is_transparent(&self) -> bool;
    fn get_description(&self) -> String;
    fn get_glyph(&self) -> char;
    fn get_color(&self) -> RGB;
}

pub struct PoirogueMap {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TileKind>,
    pub visible: Vec<bool>, // !is_transparent
    pub blocked: Vec<bool>, // !is_walkable
    pub revealed: Vec<bool>,
    pub entities: Vec<Box<dyn PoirogueEntity>>,
}

impl PoirogueMap {

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

    pub fn get_neighbors(&self, x: i32, y: i32) -> [TileKind; 4] {
        let mut result: [ TileKind; 4 ] = Default::default();
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
            tiles.push(TileKind::default());
            visible.push(false);
            revealed.push(false);
            blocked.push(true);
        }

        PoirogueMap { width: w, height: h, tiles, visible, revealed, blocked, entities: Vec::new() }
    }

    #[inline(always)]
    fn draw_tile(&self, ctx: &mut BTerm, index: usize, x: i32, y: i32) {
        let tile = &self.tiles[index];

        if self.revealed[index] {
            let color = if !self.visible[index] { tile.get_color() } else { RGB::named(GREY) };
            ctx.print_color(x, y, color, RGB::named(BLACK), tile.get_glyph());
        }
    }

    pub fn render(&self, ctx: &mut BTerm) {
        let mut index: usize = 0;

        for y in 0 .. self.height {
            for x in 0 .. self.width {
                self.draw_tile(ctx, index, x, y);
                index += 1;
            }
        }

        for entity in &self.entities {
            self.draw_entity(ctx, entity);
        }

        self.draw_entity(ctx, self.entities.get(0).unwrap());
    }

    pub fn get_player_entity_mut(&mut self) -> &mut Box<dyn PoirogueEntity> {
        self.entities.get_mut(0).unwrap()
    }

    pub fn get_player_entity(&self) -> &Box<dyn PoirogueEntity> {
        self.entities.get(0).unwrap()
    }

    pub fn draw_entity(&self, ctx: &mut BTerm, entity: &Box<dyn PoirogueEntity>) {
        let Point {x, y} = entity.get_position();
        match self.get_tile_index(x, y) {
            Some(_pos) if self.visible[_pos] =>
                ctx.print_color(x, y, entity.get_fg_color(), RGB::named(BLACK), entity.get_glyph()),
            _ => ()
        };
    }

    pub fn is_tile_walkable(&self, pos: Point) -> bool {
        match self.get_tile_index(pos.x, pos.y) {
            Some(index) => self.tiles[index].is_walkable(),
            None => false
        }
    }

    pub fn is_tile_revealed(&self, pos: Point) -> bool {
        match self.get_tile_index(pos.x, pos.y) {
            Some(index) => self.revealed[index],
            None => false
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

    pub fn set(&mut self, x: i32, y: i32, t: TileKind) {
        match self.get_tile_index(x, y) {
            Some(index) => {
                self.tiles[index] = t.clone();
                self.blocked[index] = !t.is_walkable();
                self.visible[index] = !t.is_transparent();
            },
            None => ()
        }
    }

    pub fn update_player_fov(&mut self) {
        let player_position = self.entities.get(0).unwrap().get_position();
        self.update_fov(player_position);
    }

    pub fn update_fov(&mut self, pos: Point) {
        for v in self.visible.iter_mut() {
            *v = false;
        }

        let fov = field_of_view_set(pos, 8, self);

        for index in fov.iter() {
            let point = self.point2d_to_index(*index);
            if self.tiles[point].is_obscured() {
                self.tiles[point] = TileKind::Wall;
            }
            self.visible[point] = true;
            self.revealed[point] = true;
        }
    }
}

impl BaseMap for PoirogueMap {

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

impl Algorithm2D for PoirogueMap {
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

pub trait PoirogueMapBuilder {
    fn generate(&mut self, map: &mut PoirogueMap);
}