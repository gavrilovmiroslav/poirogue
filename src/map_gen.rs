
use bracket_lib::prelude::{Bresenham, Point, RandomNumberGenerator, Rect};
use rand::Rng;
use crate::map::{FloorTiles, Map, MapTile};
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Sub};
use petgraph::{Graph, Undirected};
use petgraph::graph::NodeIndex;
use petgraph::algo::min_spanning_tree;
use petgraph::data::FromElements;

pub struct RectGenConfig {
    pub creation_attempts: i32,
    pub min_size: (i32, i32),
    pub max_size: (i32, i32),
}

trait RelativeSizing<Num>
    where Num: Add<Output=Num> + Sub<Output=Num> + Ord {
    fn bigger_by(&self, n: Num) -> Self;
    fn smaller_by(&self, n: Num) -> Self;
}

impl RelativeSizing<i32> for Rect {
    fn bigger_by(&self, n: i32) -> Self {
        Rect::with_exact(self.x1 - n, self.y1 - n, self.x2 + n, self.y2 + n)
    }

    fn smaller_by(&self, n: i32) -> Self {
        Rect::with_exact(self.x1 + n, self.y1 + n, self.x2 - n, self.y2 - n)
    }
}

type RoomIndex = usize;

pub struct MapGenStorage {
    pub rng: RandomNumberGenerator,
    pub rects: Vec<Rect>,
    pub room_index_by_rect_index: Vec<RoomIndex>,       // index in this vector = rect index by creation order;
    pub rect_center_by_rect_index: Vec<Point>,         // index in this vector = rect index by creation order;
}

impl Default for MapGenStorage {
    fn default() -> Self {
        MapGenStorage {
            rng: RandomNumberGenerator::seeded(0),
            rects: Default::default(),
            room_index_by_rect_index: Default::default(),
            rect_center_by_rect_index: Default::default(),
        }
    }
}

/*pub struct MapGenBuilder {
    pub nodes: Vec<NodeIndex>,
    pub graph: Graph<Point, i32, Undirected>
}*/

pub fn stamp_non_overlapping_rects(config: RectGenConfig, map: &mut Map, storage: &mut MapGenStorage) {
    let mut rng = &mut storage.rng;

    for _attempt in 0..config.creation_attempts {
        let width = rng.range (config.min_size.0, config.max_size.0);
        let height = rng.range(config.min_size.1, config.max_size.1);
        let x = rng.range(1, map.width - width - 1);
        let y = rng.range(1, map.height - height - 1);
        let attempt_rect = Rect::with_size(x, y, width, height);
        let insides_of_rect = attempt_rect.smaller_by(1);

        let mut okay = true;
        for stamped_rect in &storage.rects {
            if insides_of_rect.intersect(&stamped_rect) {
                okay = false;
                break;
            }
        }

        if okay {
            let index = storage.rects.len() as usize;
            storage.rects.push(attempt_rect);
            storage.room_index_by_rect_index.push(index);   // automatic new room, will re-index later
            storage.rect_center_by_rect_index.push(attempt_rect.center());

            for p in attempt_rect.point_set() {
                map.set(p.x, p.y, MapTile::Construction(index));
            }
        }
    }
}

pub fn flood_fill_construction_into_floor(map: &mut Map, storage: &mut MapGenStorage, x: i32, y: i32, fill: usize) {
    if x < 0 || y < 0 || x > map.width || y > map.height { return; }

    if let Some(index) = map.get_tile_index(x, y) {
        if let MapTile::Construction(_) = &map.tiles[index] {
            map.tiles[index] = MapTile::Floor(fill);

            flood_fill_construction_into_floor(map, storage, x - 1, y, fill);
            flood_fill_construction_into_floor(map, storage, x, y - 1, fill);
            flood_fill_construction_into_floor(map, storage, x, y + 1, fill);
            flood_fill_construction_into_floor(map, storage, x + 1, y, fill);
        }
    }
}

pub fn link_rects_into_rooms(map: &mut Map, storage: &mut MapGenStorage) {
    let mut index = 0;
    for i in 1..map.width - 1 {
        for j in 1..map.height - 1 {
            let tile = map.get_tile_index(i, j).unwrap();
            if let MapTile::Construction(n) = map.tiles[tile] {
                storage.room_index_by_rect_index[n] = index;

                flood_fill_construction_into_floor(map, storage, i, j, index);
                index += 1;
            }
        }
    }
}

/*
pub fn link_neighbors(map: &mut Map, rooms: &RectsInRoom, graph: &mut Graph<>) {
    let mut rng = rand::thread_rng();
    let mut index_rect_by_center = HashMap::new();

    for i in 0..map.rooms.len() {
        let rect = &map.rooms[i];
        let center = rect.center();
        index_rect_by_center.insert(center, rect);

        let scope = Rect::with_size(rect.x1 - 10, rect.y1 - 10, 20, 20);

        for j in 0..map.rooms.len() {
            if map.rect_in_room[j] == map.rect_in_room[i] { continue; }

            let other = &map.rooms[j];
            if scope.point_in_rect(other.center()) {
             //   graph.add_edge(graph.nodes[i], self.nodes[j], rng.gen_range(1..100));
            }
        }
    }

    let mst = min_spanning_tree(&self.graph);
    let graph = Graph::<Point, i32, Undirected>::from_elements(mst);
    for x in graph.raw_edges() {
        let a = graph.node_weight(x.source()).unwrap();
        let b = graph.node_weight(x.target()).unwrap();
        //self.draw_corridor(map, *a, *b);
    }
}*/
/*
fn draw_corridor(&mut self, map: &mut Map, p: Point, q: Point) {
    let line: Vec<Point> = Bresenham::new(p, q).collect();
    for c in line {
        let index = map.get_tile_index(c.x, c.y).unwrap();
        match map.tiles[index] {
            MapTile::Obscured  => { map.set(c.x, c.y, MapTile::Corridor); }
            _ => {}
        }
    }
}

fn get_random_room_center(&self) -> &Point {
    let mut rng = rand::thread_rng();
    let g = self.nodes.get(rng.gen_range(0..self.nodes.len())).unwrap();

    self.graph.node_weight(*g).unwrap()
}*/

pub fn run_map_gen(w: i32, h: i32) -> Map {
    let mut map = Map::new(w, h);
    let mut storage = MapGenStorage::default();

    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 3, min_size: (10, 8), max_size: (12, 12) }, &mut map, &mut storage);
    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 200, min_size: (5, 5), max_size: (8, 7) }, &mut map, &mut storage);
    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 300, min_size: (4, 2), max_size: (6, 5) }, &mut map, &mut storage);
    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 500, min_size: (2, 3), max_size: (4, 4) }, &mut map, &mut storage);
    link_rects_into_rooms(&mut map, &mut storage);

//        let mut graph = Graph::new_undirected();
    //link_neighbors(&mut map, &rooms, &mut graph);

    map
}