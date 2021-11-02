use bracket_lib::prelude::{a_star_search, BTerm, GameState, Point, RGBA, VirtualKeyCode};

use crate::entity::PoirogueEntity;
use crate::game::ticks::{TickType, TurnType};
use crate::map::{PoirogueMap};

pub trait TickStates : Default + 'static {}

pub enum Command {
    MoveTo(Point),
    EndTurn,
}

pub struct PoirogueGameManager {
    pub tick: TickType,
    pub map: PoirogueMap,
    pub planned_path: Vec<Point>,
    pub frame_count: u64,
}

impl GameState for PoirogueGameManager {

    fn tick(&mut self, ctx: &mut BTerm) {
        self.update(ctx);
        ctx.cls();
        self.render(ctx);
    }
}

impl PoirogueGameManager {

    pub fn new(w: i32, h: i32) -> Self {
        let tick = TickType::default();
        let map: PoirogueMap = PoirogueMap::new(w, h);

        PoirogueGameManager { tick, map, planned_path: vec![], frame_count: 0 }
    }

    pub fn render(&mut self, ctx: &mut BTerm) {
        self.frame_count += 1;

        if self.frame_count < 2 { return; }

        self.map.render(ctx);

        for p in &self.planned_path {
            ctx.set_bg(p.x, p.y,RGBA::from_f32(1.0, 1.0, 1.0, 0.5));
        }

        let mouse = ctx.mouse_pos();
        ctx.set_bg(mouse.0, mouse.1, RGBA::from_f32(1.0, 1.0, 1.0, 0.5));
    }

    fn get_player_point(&self) -> Point {
        self.map.get_player_entity().get_position()
    }

    fn move_player_to(&mut self, x: i32, y: i32) {
        match self.map.entities.get(0) {
            Some(player) => {
                let destination = Point { x, y };
                if player.get_position() != destination {
                    let next_step = self.plan_path(destination);

                    if self.map.is_tile_walkable(next_step) {
                        self.map.entities.get_mut(0).unwrap().set_position(next_step);
                    }

                    let player_mut = self.map.get_player_entity_mut();
                    player_mut.add_command(Command::MoveTo(destination));
                    player_mut.add_command(Command::EndTurn);
                }
            },
            None => {}
        }
    }

    fn move_player_by(&mut self, dx: i32, dy: i32) {
        match self.map.entities.get(0) {
            Some(player) => {
                let position = player.get_position();
                let next_position = Point::new(position.x + dx, position.y + dy);
                if self.map.is_tile_walkable(next_position) {
                    self.map.entities.get_mut(0).unwrap().set_position(next_position);
                }
            },
            None => {}
        }

        self.tick = TickType::InGame(TurnType::WorldTurn);
    }

    fn end_turn(&mut self) {
        self.tick = TickType::InGame(TurnType::WorldTurn);
    }

    fn plan_path(&self, c: Point) -> Point {
        let p = self.get_player_point();
        let index = self.map.get_tile_index(c.x, c.y).unwrap();

        if self.map.is_tile_walkable(c) && self.map.is_tile_revealed(c) {
            let path = a_star_search(self.map.get_tile_index(p.x, p.y).unwrap(), index, &self.map);

            if path.success {
                let loc = path.steps.iter().skip(1).next().unwrap();
                return Point::from_tuple(self.map.get_tile_coords(*loc));
            }
        }

        return p;
    }

    fn manual_input(&mut self, ctx: &mut BTerm) {
        if ctx.left_click {
            let cursor = ctx.mouse_point();
            if self.map.is_tile_revealed(cursor) && self.map.is_tile_walkable(cursor) {
                self.plan_path(cursor);

                let player = self.map.get_player_entity_mut();
                player.add_command(Command::MoveTo(cursor));
                self.end_turn();
            }
        }

        match ctx.key {
            Some(VirtualKeyCode::W) => self.move_player_by(0, -1),
            Some(VirtualKeyCode::D) => self.move_player_by(1, 0),
            Some(VirtualKeyCode::A) => self.move_player_by(-1, 0),
            Some(VirtualKeyCode::S) => self.move_player_by(0, 1),
            Some(VirtualKeyCode::Q) => self.move_player_by(-1, -1),
            Some(VirtualKeyCode::E) => self.move_player_by(1, -1),
            Some(VirtualKeyCode::Z) => self.move_player_by(-1, 1),
            Some(VirtualKeyCode::C) => self.move_player_by(1, 1),
            _ => {}
        }
    }

    pub fn update(&mut self, ctx: &mut BTerm) {
        if ctx.key == Some(VirtualKeyCode::Escape) {
            self.map.get_player_entity_mut().clear_commands();
        }

        match self.tick {
            TickType::None => self.end_turn(),

            TickType::InGame(TurnType::PlayerTurn) => {
                match self.map.get_player_entity_mut().get_next_command() {
                    Some(Command::MoveTo(Point{x, y})) => {
                        if ctx.key == Some(VirtualKeyCode::Escape) { return; }
                        self.move_player_to(x, y)
                    },
                    Some(Command::EndTurn) => self.end_turn(),
                    _ => self.manual_input(ctx)
                }
            }

            TickType::InGame(TurnType::WorldTurn) => {
                self.map.update_player_fov();
                self.tick = TickType::InGame(TurnType::PlayerTurn);
            }
            _ => {}
        }
    }
}
