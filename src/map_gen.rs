use serde::{Serialize, Deserialize};
use bracket_lib::prelude::{Algorithm2D, BaseMap, Bresenham, BresenhamCircle, DijkstraMap, line2d_bresenham, Point, RandomNumberGenerator, Rect, VectorLine};
use crate::rand_gen::{get_random_between};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::ops::{Add, Range, Sub};
use multimap::MultiMap;
use urlencoding::encode;
use crate::map::Map;
use crate::tiles::{DebugMapTile, RectIndex, RoomIndex, TileIndex, MapTile, DoorState};

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
    pub rects_in_room: MultiMap<RoomIndex, Rect>,
    pub door_tiles: Vec<TileIndex>,
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
        if let MapTile::Debug(DebugMapTile::Construction(old_room_index)) = &map.get_tile_at(tile_index) {
            let rect_index = storage.lookup_rect_by_tile_index.get(&tile_index).unwrap();

            map.set_at_tile_index(tile_index, MapTile::Floor(fill_room_index));
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
    let mut room_index = 0;
    for i in 1..map.width - 1 {
        for j in 1..map.height - 1 {
            let tile = map.get_tile_index(i, j).unwrap();
            if let MapTile::Debug(DebugMapTile::Construction(old_room_index)) = map.get_tile_at(tile) {
                storage.room_index_by_rect_index[*old_room_index] = room_index;

                insert_rect_to_room(storage, *old_room_index, room_index);

                flood_fill_construction_into_floor(map, storage, i, j, room_index);
                room_index += 1;
            }
        }
    }
}

#[derive(PartialEq)]
enum Dir { LeftRight, UpDown }

pub fn link_neighbors(map: &mut Map, storage: &mut MapGenStorage) {

    fn connect_by_axis(map: &mut Map, storage: &mut MapGenStorage, rect_index: RectIndex, max_dist: i32, axis: Dir) {
        let movement = match axis {
            Dir::LeftRight => |p: Point| Point::new(p.x + 1, p.y),
            Dir::UpDown => |p: Point| Point::new(p.x, p.y + 1),
        };

        let range = match axis {
            Dir::LeftRight => |rect: Rect| rect.y1..rect.y2,
            Dir::UpDown => |rect: Rect| rect.x1..rect.x2,
        };

        let side = match axis {
            Dir::LeftRight => |rect: Rect, h: i32| Point::new(rect.x2, h),
            Dir::UpDown => |rect: Rect, w: i32| Point::new(w, rect.y2),
        };

        let checker = |axis: Dir| {
            let hor = axis == Dir::LeftRight;

            move |map: &Map, p: Point| {
                let prev_tile_index = map.point2d_to_index(if hor { Point::new(p.x, p.y - 1) } else { Point::new(p.x - 1, p.y) });
                let next_tile_index = map.point2d_to_index(if hor { Point::new(p.x, p.y + 1) } else { Point::new(p.x + 1, p.y) });
                let prev_tile = map.get_tile_at(prev_tile_index);
                let next_tile = map.get_tile_at(next_tile_index);
                prev_tile.is_obscured() && next_tile.is_obscured()
            }
        };

        fn connect_by_axis_internal<MovePoint, PointRange, PointExtreme, HallwayChecker>
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
                    map.set_at_tile_index(tile, MapTile::Corridor);
                    if checker(map, map.index_to_point2d(tile)) {
                        door_candidates.push(tile);
                    }
                }

                if !door_candidates.is_empty() {
                    let door = door_candidates[get_random_between(0, door_candidates.len())];
                    map.set_at_tile_index(door, MapTile::Door(DoorState::Closed));
                    storage.door_tiles.push(door);
                }
            }
        }

        connect_by_axis_internal(map, storage, rect_index, max_dist, movement, range, side, checker(axis));
    }

    for i in 0..storage.rects.len() {
        connect_by_axis(map, storage, i, 5, Dir::LeftRight);
        connect_by_axis(map, storage, i, 3, Dir::UpDown);
    }
}

pub fn remove_weird_doors(map: &mut Map, storage: &mut MapGenStorage) {
    const LEFT: i32 = 0;
    const RIGHT: i32 = 1;
    const UP: i32 = 2;
    const DOWN: i32 = 3;

    fn check_door_between_two_rooms(map: &Map, neighbors: [Point; 4], main_dir: Dir) -> bool {
        let dir = if main_dir == Dir::LeftRight { 0 } else { 2 };

        is_room(map, neighbors[dir]) &&
            is_room(map, neighbors[dir + 1]) &&
            (is_corridor_or_door(map, neighbors[(dir + 2) % 4]) ||
                is_corridor_or_door(map, neighbors[(dir + 3) % 4]))
    }

    fn neighbors(pt: Point) -> [(Point, i32); 4] {
        [
            (pt + Point::new(-1, 0), 0),
            (pt + Point::new(1, 0), 0),
            (pt + Point::new(0, -1), 1),
            (pt + Point::new(0, 1), 1),
        ]
    }

    fn is_corridor_or_door(map: &Map, pt: Point) -> bool {
        let tile = map.point2d_to_index(pt);
        map.is_tile(tile, MapTile::Corridor) ||
            map.is_tile(tile, MapTile::Door(DoorState::Closed)) ||
            map.is_tile(tile, MapTile::Door(DoorState::Open))
    }

    fn is_room(map: &Map, pt: Point) -> bool {
        let tile = map.point2d_to_index(pt);
        let is_room = if let MapTile::Floor(_) = map.get_tile_at(tile) { true } else { false };
        is_room
    }

    fn remove_doors_if_too_wide(map: &mut Map, storage: &mut MapGenStorage) {
        for &door in &storage.door_tiles {
            let pt = map.index_to_point2d(door);
            if map.in_bounds(pt) {
                let mut count = [0, 0];

                for (neighbor, axis) in neighbors(pt) {
                    if map.in_bounds(neighbor) {
                        if is_corridor_or_door(map, neighbor) {
                            count[axis as usize] += 1;
                        }
                    }
                }

                if count[0] != 0 && count[1] != 0 {
                    map.set_at_tile_index(door, MapTile::Corridor);
                }
            }
        }
    }

    fn remove_doors_if_crowded(map: &mut Map, storage: &mut MapGenStorage) {
        for &door in &storage.door_tiles {
            let pt = map.index_to_point2d(door);
            if map.in_bounds(pt) {
                let mut done = false;
                let neighs = neighbors(pt);
                for (n, axis) in neighs {
                    if !map.in_bounds(n) {
                        map.set_at_tile_index(door, MapTile::Corridor);
                        done = true;
                        break;
                    }
                }

                if done { continue; }
                let neighbors = neighbors(pt).map(|(a, _)| a);

                if check_door_between_two_rooms(map, neighbors, Dir::LeftRight) ||
                    check_door_between_two_rooms(map, neighbors, Dir::UpDown) {
                    map.set_at_tile_index(door, MapTile::Corridor);
                }
            }
        }
    }

    remove_doors_if_too_wide(map, storage);
    remove_doors_if_crowded(map, storage);
}

pub fn run_map_gen(w: i32, h: i32) -> (Map, MapGenStorage) {
    let mut map = Map::new(w, h);
    let mut storage = MapGenStorage::default();

    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 3, min_size: (10, 8), max_size: (12, 12) }, &mut map, &mut storage);
    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 200, min_size: (5, 5), max_size: (8, 7) }, &mut map, &mut storage);
    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 300, min_size: (4, 3), max_size: (6, 5) }, &mut map, &mut storage);
    stamp_non_overlapping_rects(RectGenConfig { creation_attempts: 50, min_size: (3, 3), max_size: (4, 4) }, &mut map, &mut storage);
    glue_rects_into_rooms(&mut map, &mut storage);

    link_neighbors(&mut map, &mut storage);
    remove_weird_doors(&mut map, &mut storage);

    (map, storage)
}