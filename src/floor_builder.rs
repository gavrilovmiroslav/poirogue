
use bracket_lib::prelude::{Bresenham, Point, Rect};
use rand::Rng;
use crate::map::{FloorTiles, PoirogueMap, PoirogueTile};
use std::collections::{HashMap};
use petgraph::{Graph, Undirected};
use petgraph::graph::NodeIndex;
use petgraph::algo::min_spanning_tree;
use petgraph::data::FromElements;
use crate::pawn::Pawn;

pub struct RoomBuilderConfig {
    pub room_creation_attempts: i32,
    pub min_room_size: (i32, i32),
    pub max_room_size: (i32, i32),
}

pub struct FloorMapBuilder {
    pub rects: Vec<Rect>,
    pub rect_in_room: Vec<u8>,
    pub rooms: HashMap<u8, Vec<usize>>,
    pub nodes: Vec<NodeIndex>,
    pub graph: Graph<Point, i32, Undirected>
}

impl FloorMapBuilder {

    pub fn new() -> FloorMapBuilder {
        FloorMapBuilder {
            rects: Vec::new(),
            rooms: HashMap::new(),
            rect_in_room: Vec::new(),
            nodes: Vec::new(),
            graph: Graph::new_undirected()
        }
    }

    fn fill_rooms(&mut self, config: RoomBuilderConfig, map: &mut PoirogueMap) {
        let mut rng = rand::thread_rng();

        for _attempt in 0..config.room_creation_attempts {
            let room_width = rng.gen_range(config.min_room_size.0..config.max_room_size.0);
            let room_height = rng.gen_range(config.min_room_size.1..config.max_room_size.1);
            let room_x = rng.gen_range(1..map.width - room_width - 1);
            let room_y = rng.gen_range(1..map.height - room_height - 1);
            let room = Rect::with_size(room_x, room_y, room_width, room_height);
            let small_room = Rect::with_size(room_x + 1, room_y + 1, room_width - 1, room_height - 1);

            let mut okay = true;
            for other in &self.rects {
                if small_room.intersect(other) {
                    okay = false;
                    break;
                }
            }

            if okay {
                let index = self.rects.len() as u8;
                self.rects.push(room);
                for p in room.point_set() {
                    map.set(p.x, p.y, PoirogueTile::RectRoom(index));
                }

                self.rect_in_room.push(index);
                let c = room.center();
                let node = self.graph.add_node(c);
                self.nodes.push(node);
            }
        }
    }

    fn link_rects_into_rooms(&mut self, map: &mut PoirogueMap) {

        #[inline(always)]
        fn flood_fill(builder: &mut FloorMapBuilder, map: &mut PoirogueMap, x: i32, y: i32, fill: u8) {
            if x < 0 || y < 0 || x > map.width || y > map.height { return; }

            let try_index = map.get_tile_index(x, y);
            if try_index.is_none() { return; }

            let index = try_index.unwrap();

            match &map.tiles[index] {
                PoirogueTile::RectRoom(_) => {
                    let is_internal = !map.get_neighbors(x, y).map(|x| x.is_wall()).contains(&true);
                    let internal_tile = if is_internal { FloorTiles::Internal } else { FloorTiles::Edge };
                    map.tiles[index] = PoirogueTile::Floor(fill, internal_tile);
                    builder.rooms.get_mut(&fill).unwrap().push(index);

                    flood_fill(builder, map, x - 1, y, fill);
                    flood_fill(builder, map, x, y - 1, fill);
                    flood_fill(builder, map, x, y + 1, fill);
                    flood_fill(builder, map, x + 1, y, fill);
                },

                _ => {}
            }
        }

        let mut index = 0;
        for i in 1..map.width - 1 {
            for j in 1..map.height - 1 {
                let tile = map.get_tile_index(i, j).unwrap();
                match map.tiles[tile] {
                    PoirogueTile::RectRoom(n) => {
                        self.rect_in_room[n as usize] = index;
                        if !self.rooms.contains_key(&index) {
                            self.rooms.insert(index, Vec::new());
                        }
                        flood_fill(self, map, i, j, index);
                        index += 1;
                    },
                    _ => ()
                }
            }
        }
    }

    fn link_neighbors(&mut self, map: &mut PoirogueMap) {
        let mut rng = rand::thread_rng();

        for i in 0..self.rects.len() {
            let rect = &self.rects[i];
            let scope = Rect::with_size(rect.x1 - 10, rect.y1 - 10, 20, 20);

            for j in 0..self.rects.len() {
                if self.rect_in_room[j] == self.rect_in_room[i] { continue; }

                let other = &self.rects[j];
                if scope.point_in_rect(other.center()) {
                    self.graph.add_edge(self.nodes[i], self.nodes[j], rng.gen_range(1..100));
                }
            }
        }

        let mst = min_spanning_tree(&self.graph);
        let graph = Graph::<Point, i32, Undirected>::from_elements(mst);
        for x in graph.raw_edges() {
            let a = graph.node_weight(x.source()).unwrap();
            let b = graph.node_weight(x.target()).unwrap();
            self.draw_corridor(map, *a, *b);
        }
    }

    fn draw_corridor(&mut self, map: &mut PoirogueMap, p: Point, q: Point) {
        let line: Vec<Point> = Bresenham::new(p, q).collect();
        for c in line {
            let index = map.get_tile_index(c.x, c.y).unwrap();
            match map.tiles[index] {
                PoirogueTile::Obscured(_)  => { map.set(c.x, c.y, PoirogueTile::Corridor); }
                _ => {}
            }
        }
    }

    fn get_random_room_center(&self) -> &Point {
        let mut rng = rand::thread_rng();
        let g = self.nodes.get(rng.gen_range(0..self.nodes.len())).unwrap();

        self.graph.node_weight(*g).unwrap()
    }

    fn place_objects(&mut self, map: &mut PoirogueMap) {
        let mut rng = rand::thread_rng();
        for rect in &self.rects {
            let mut center = rect.center();
            center.x += rng.gen_range(-2..2);
            center.y += rng.gen_range(-2..2);
//            map.entities.push(Box::new(Mob::new(center)));
        }
    }

    pub fn generate(&mut self, map: &mut PoirogueMap) {
        self.fill_rooms(RoomBuilderConfig { room_creation_attempts: 10, min_room_size: (10, 8), max_room_size: (15, 12) }, map);
        self.fill_rooms(RoomBuilderConfig { room_creation_attempts: 200, min_room_size: (5, 5), max_room_size: (8, 7) }, map);
        self.fill_rooms(RoomBuilderConfig { room_creation_attempts: 300, min_room_size: (4, 4), max_room_size: (6, 5) }, map);
        self.link_rects_into_rooms(map);
        self.link_neighbors(map);

        map.entities.push(Pawn::new_player(*self.get_random_room_center()));
        self.place_objects(map);
    }
}