use serde::{Serialize, Deserialize};
use bracket_lib::prelude::{Algorithm2D, BaseMap, Bresenham, BresenhamCircle, DijkstraMap, line2d_bresenham, Point, RandomNumberGenerator, Rect, VectorLine};
use crate::rand_gen::{get_random_between};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::ops::{Add, Range, Sub};
use multimap::MultiMap;
use petgraph::{Graph, Undirected};
use petgraph::graph::NodeIndex;
use petgraph::algo::min_spanning_tree;
use petgraph::data::FromElements;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::EdgeRef;
use petgraph::visit::NodeRef;
use urlencoding::encode;
use crate::map::Map;
use crate::tiles::{DebugMapTile, RectIndex, RoomIndex, TileIndex, MapTile};

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

#[derive(Default)]
pub struct MapGenStorage {
    pub rects: Vec<Rect>,
    pub room_index_by_rect_index: Vec<RoomIndex>,       // index in this vector = rect index by creation order;
    pub rect_center_by_rect_index: Vec<Point>,          // index in this vector = rect index by creation order;
    pub lookup_rect_by_tile_index: HashMap<TileIndex, RectIndex>,
    pub room_graph: Graph<RoomIndex, f32>,
    pub node_index_by_rect_index: HashMap<RoomIndex, NodeIndex>,
    pub rects_in_room: MultiMap<RoomIndex, Rect>,
    pub edge_tiles_in_room: MultiMap<RoomIndex, TileIndex>,
    pub edge_tiles_in_rect: MultiMap<RectIndex, TileIndex>,
}

impl MapGenStorage {
    pub fn are_rects_in_same_room(&self, a: RectIndex, b: RectIndex) -> bool {
        let room_a = self.room_index_by_rect_index[a];
        let room_b = self.room_index_by_rect_index[b];
        room_a == room_b
    }

    pub fn are_tiles_in_same_room(&self, a: TileIndex, b: TileIndex) -> bool {
        let rect_a = self.lookup_rect_by_tile_index.get(&a).unwrap();
        let rect_b = self.lookup_rect_by_tile_index.get(&b).unwrap();
        let room_a = self.room_index_by_rect_index[*rect_a];
        let room_b = self.room_index_by_rect_index[*rect_b];
        room_a == room_b
    }

    pub fn get_graph_node_from_tile(&self, t: TileIndex) -> NodeIndex {
        let index = self.room_index_by_rect_index[self.lookup_rect_by_tile_index[&t]];
        self.node_index_by_rect_index[&index]
    }
}


pub fn stamp_non_overlapping_rects(config: RectGenConfig, map: &mut Map, storage: &mut MapGenStorage) {
    for _attempt in 0..config.creation_attempts {
        let width = get_random_between(config.min_size.0, config.max_size.0);
        let height = get_random_between(config.min_size.1, config.max_size.1);
        let x = get_random_between(1, map.width - width - 1);
        let y = get_random_between(1, map.height - height - 1);
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
            let index = storage.rects.len() as RoomIndex;
            storage.rects.push(attempt_rect);
            storage.room_index_by_rect_index.push(index);   // automatic new room, will re-index later
            storage.rect_center_by_rect_index.push(attempt_rect.center());

            for p in attempt_rect.point_set() {
                map.set(p.x, p.y, MapTile::Debug(DebugMapTile::Construction(index)));
                storage.lookup_rect_by_tile_index.insert(map.get_tile_index(p.x, p.y).unwrap(), index);
            }
        }
    }
}

fn insert_rect_to_room(storage: &mut MapGenStorage, rect_index: RectIndex, room_index: RoomIndex) {
    let rect = storage.rects[rect_index];
    if let Some(vec) = storage.rects_in_room.get_vec(&room_index) {
        if !vec.contains(&rect) {
            storage.rects_in_room.insert(room_index, rect);
        }
    }
    else
    {
        storage.rects_in_room.insert(room_index, rect);
    }
}

pub fn flood_fill_construction_into_floor(map: &mut Map, storage: &mut MapGenStorage, x: i32, y: i32, fill_room_index: RoomIndex) {
    if x < 0 || y < 0 || x > map.width || y > map.height { return; }

    if let Some(tile_index) = map.get_tile_index(x, y) {
        if let MapTile::Debug(DebugMapTile::Construction(old_room_index)) = &map.tiles[tile_index] {
            let rect_index = storage.lookup_rect_by_tile_index.get(&tile_index).unwrap();

            map.tiles[tile_index] = MapTile::Floor(fill_room_index);
            *storage.room_index_by_rect_index.get_mut(*rect_index).unwrap() = fill_room_index;

            insert_rect_to_room(storage, *rect_index, fill_room_index);

            flood_fill_construction_into_floor(map, storage, x - 1, y, fill_room_index);
            flood_fill_construction_into_floor(map, storage, x, y - 1, fill_room_index);
            flood_fill_construction_into_floor(map, storage, x, y + 1, fill_room_index);
            flood_fill_construction_into_floor(map, storage, x + 1, y, fill_room_index);
        }
    }
}

pub fn glue_rects_into_rooms(map: &mut Map, storage: &mut MapGenStorage) {
    fn points_diagonal_from(p: &Point) -> [Point;4] {
        [
            Point::new(p.x - 1, p.y - 1),
            Point::new(p.x - 1, p.y + 1),
            Point::new(p.x + 1, p.y - 1),
            Point::new(p.x + 1, p.y + 1),
        ]
    }

    let mut room_index = 0;
    for i in 1..map.width - 1 {
        for j in 1..map.height - 1 {
            let tile = map.get_tile_index(i, j).unwrap();
            if let MapTile::Debug(DebugMapTile::Construction(old_room_index)) = map.tiles[tile] {
                storage.room_index_by_rect_index[old_room_index] = room_index;

                insert_rect_to_room(storage, old_room_index, room_index);

                flood_fill_construction_into_floor(map, storage, i, j, room_index);
                room_index += 1;
            }
        }
    }

    for (room, rects) in &storage.rects_in_room {
        let union = rects.iter()
            .fold(HashSet::new(), |a, &r| {
                let rps = &r.point_set();
                a.union(&rps).copied().collect()
            });

        let edges = rects.iter()
            .fold(HashSet::<Point>::new(), |a, r| {
                let small = r.smaller_by(1);
                let points = r.point_set();
                let smaller_points = small.point_set();
                let diff = points.difference(&smaller_points).map(|r| *r).collect();
                a.union(&diff).map(|r| *r).collect()
            });

        edges.iter().for_each(|p| {
            let out = points_diagonal_from(p).iter().fold(0, |a, p| {
                if !union.contains(p) { a + 1 } else { a }
            });

            if out != 0 {
                if let Some(index) = map.get_tile_index_from_point(*p) {
                    let rect = storage.lookup_rect_by_tile_index.get(&index).unwrap();
                    storage.edge_tiles_in_room.insert(*room, index);
                    storage.edge_tiles_in_rect.insert(*rect, index);
                    //map.tiles[index] = MapTile::Corridor;
                }
            }
        })
    }
}

pub fn link_neighbors(map: &mut Map, storage: &mut MapGenStorage) {
    fn connect_axially<MovePoint, PointRange, PointExtreme, HallwayChecker>
            (map: &mut Map, storage: &mut MapGenStorage, rect_index: RectIndex, max_dist: i32,
             movement: MovePoint, range: PointRange, side: PointExtreme, checker: HallwayChecker)
        where
            MovePoint: Fn(Point) -> Point,
            PointRange: Fn(Rect) -> Range<i32>,
            PointExtreme: Fn(Rect, i32) -> Point,
            HallwayChecker: Fn(&Map, Point) -> bool {

        let mut correct_paths = Vec::new();
        let rect = storage.rects[rect_index];

        for h in range(rect) {
            let mut path = Vec::new();
            let mut target = side(rect, h);
            let home_room = storage.room_index_by_rect_index[rect_index];

            for _ in 0..max_dist {
                if let Some(tile) = map.get_tile_index_from_point(target) {
                    if let Some(&other_rect) = storage.lookup_rect_by_tile_index.get(&tile) {
                        if other_rect == rect_index { continue; }

                        let other_room = storage.room_index_by_rect_index[other_rect];
                        if other_room != home_room && !path.is_empty() {
                            correct_paths.push(path);
                        }
                        break;
                    } else {
                        path.push(tile);
                    }
                } else {
                    break;
                }

                target = movement(target);
            }
        }

        if !correct_paths.is_empty() {
            let mut door_candidates = Vec::new();

            let random = get_random_between(0, correct_paths.len());
            let path = &correct_paths[random];
            let len = path.len();

            for i in 0..len {
                let tile = path[i];
                map.tiles[tile] = MapTile::Corridor;
                if checker(map, map.index_to_point2d(tile)) {
                    door_candidates.push(tile);
                }
            }

            if !door_candidates.is_empty() {
                let door = door_candidates[get_random_between(0, door_candidates.len())];
                map.tiles[door] = MapTile::Door;
            }
        }
    }

    for i in 0..storage.rects.len() {
        connect_axially(map, storage, i, 5,
                        |p| Point::new(p.x + 1, p.y),
                           |rect| rect.y1..rect.y2,
                            |rect, h| Point::new(rect.x2, h),
                          |map, p| {
                              let above = map.point2d_to_index(Point::new(p.x, p.y - 1));
                              let below = map.point2d_to_index(Point::new(p.x, p.y + 1));
                              map.tiles[above] == MapTile::Obscured && map.tiles[below] == MapTile::Obscured
                          });

        connect_axially(map, storage, i, 3,
                        |p| Point::new(p.x, p.y + 1),
                        |rect| rect.x1..rect.x2,
                        |rect, w| Point::new(w, rect.y2),
                        |map, p| {
                            let left = map.point2d_to_index(Point::new(p.x - 1, p.y));
                            let right = map.point2d_to_index(Point::new(p.x + 1, p.y));
                            map.tiles[left] == MapTile::Obscured && map.tiles[right] == MapTile::Obscured
                        });
    }
}

pub fn run_map_gen(w: i32, h: i32) -> Map {
    let mut map = Map::new(w, h);
    let mut storage = MapGenStorage::default();

    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 3, min_size: (10, 8), max_size: (12, 12) }, &mut map, &mut storage);
    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 200, min_size: (5, 5), max_size: (8, 7) }, &mut map, &mut storage);
    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 300, min_size: (4, 3), max_size: (6, 5) }, &mut map, &mut storage);
    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 50, min_size: (3, 3), max_size: (4, 4) }, &mut map, &mut storage);
    glue_rects_into_rooms(&mut map, &mut storage);

    link_neighbors(&mut map, &mut storage);

    map
}