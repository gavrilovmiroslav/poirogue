use std::borrow::Borrow;
use serde::{Serialize, Deserialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::AddAssign;
use std::rc::Rc;
use bracket_color::prelude::{RGB, BLACK, WHITE};
use bracket_lib::prelude::{Algorithm2D, field_of_view_set, Point, VirtualKeyCode};
use crate::colors::named_color;
use crate::commands::ActionCommand;
use crate::entity::Entity::{Character, Player};
use crate::game::GameSharedData;
use crate::glyph::{Glyph};
use crate::input::InputSnapshot;

pub type InMut<T> = Rc<RefCell<T>>;

pub trait AbstractEntity {
    type Data;

    fn inner_mut(&mut self) -> &mut Transform;
    fn inner(&self) -> &Transform;

    fn get_position(&self) -> Point;
    fn get_glyph(&self) -> Glyph;

    fn get_fov(&self) -> HashSet<Point>;

    fn is_player(&self) -> bool;
    fn tick(&self, data: &Self::Data) -> Vec<ActionCommand>;
}

pub struct Transform {
    pub position: Point,
    pub glyph: Glyph,
    pub fov: HashSet<Point>,
}

pub enum Entity {
    Player(Transform),
    Character(Transform),
}

impl Entity {
    pub fn make_player(pos: Point) -> InMut<Entity> {
        Rc::new(RefCell::new(Player(Transform{
            position: pos,
            glyph: Glyph {
                ch: '@',
                fg: named_color(WHITE),
                bg: named_color(BLACK) },
            fov: HashSet::default(),
        })))
    }

    pub fn make_character(pos: Point, ch: char) -> InMut<Entity> {
        Rc::new(RefCell::new(Character(Transform{
            position: pos,
            glyph: Glyph {
                ch,
                fg: named_color(WHITE),
                bg: named_color(BLACK) },
            fov: HashSet::default(),
        })))
    }
}

impl AbstractEntity for Entity {
    type Data = GameSharedData;

    fn inner_mut(&mut self) -> &mut Transform {
        match self {
            Player(t) => t,
            Character(t) => t,
        }
    }

    fn inner(&self) -> &Transform {
        match self {
            Player(t) => t,
            Character(t) => t,
        }
    }

    fn get_position(&self) -> Point {
        self.inner().position
    }
    fn get_glyph(&self) -> Glyph {
        self.inner().glyph
    }
    fn get_fov(&self) -> HashSet<Point> {
        self.inner().fov.clone()
    }

    fn is_player(&self) -> bool {
        if let Player(_) = self { true } else { false }
    }

    fn tick(&self, data: &Self::Data) -> Vec<ActionCommand> {
        let mut result = Vec::new();

        if self.is_player() {
            let fov = field_of_view_set(self.get_position(), data.store.get("fov").unwrap_or(16), &data.map)
                .iter().map(|p| (p.x, p.y)).collect();
            result.push(ActionCommand::FovChange(fov));

            if data.input.keyboard.is_pressed_or_held_with_mod(VirtualKeyCode::Up, VirtualKeyCode::LShift) {
                result.push(ActionCommand::MoveBy(0, -1));
            }

            if data.input.keyboard.is_pressed_or_held_with_mod(VirtualKeyCode::Down, VirtualKeyCode::LShift) {
                result.push(ActionCommand::MoveBy(0, 1));
            }

            if data.input.keyboard.is_pressed_or_held_with_mod(VirtualKeyCode::Left, VirtualKeyCode::LShift) {
                result.push(ActionCommand::MoveBy(-1, 0));
            }

            if data.input.keyboard.is_pressed_or_held_with_mod(VirtualKeyCode::Right, VirtualKeyCode::LShift) {
                result.push(ActionCommand::MoveBy(1, 0));
            }
        }

        result
    }
}
