use serde::{Serialize, Deserialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::AddAssign;
use std::rc::Rc;
use bracket_color::prelude::{RGB, BLACK, WHITE};
use bracket_lib::prelude::{Algorithm2D, field_of_view_set, Point};
use crate::entity::Entity::{Character, Player};
use crate::game::GameSharedData;
use crate::glyph::{Glyph};

pub type InMut<T> = Rc<RefCell<T>>;

pub trait AbstractEntity {
    type Data;

    fn move_by(&mut self, dist: Point);
    fn move_to(&mut self, dist: Point);
    fn get_position(&self) -> Point;

    fn set_glyph(&mut self, glyph: Glyph);
    fn set_glyph_ch(&mut self, ch: char);
    fn set_glyph_fg(&mut self, fg: RGB);
    fn set_glyph_bg(&mut self, bg: RGB);
    fn get_glyph(&self) -> Glyph;

    fn get_fov(&self) -> HashSet<Point>;

    fn is_player(&self) -> bool;
    fn is_dirty(&self) -> bool;
    fn mark_dirty(&mut self);

    fn tick(&mut self, data: &Self::Data);
}

pub struct Transform {
    pub position: Point,
    pub glyph: Glyph,
    pub fov: HashSet<Point>,
}

pub enum Entity {
    Player(Transform, bool),
    Character(Transform),
}

impl Entity {
    pub fn make_player(pos: Point) -> InMut<Entity> {
        Rc::new(RefCell::new(Player(Transform{
            position: pos,
            glyph: Glyph {
                ch: '@',
                fg: RGB::named(WHITE),
                bg: RGB::named(BLACK) },
            fov: HashSet::default()
        }, true)))
    }

    pub fn make_character(pos: Point, ch: char) -> InMut<Entity> {
        Rc::new(RefCell::new(Character(Transform{
            position: pos,
            glyph: Glyph {
                ch,
                fg: RGB::named(WHITE),
                bg: RGB::named(BLACK) },
            fov: HashSet::default()
        })))
    }
}

impl Entity {
    pub fn inner_mut(&mut self) -> &mut Transform {
        match self {
            Player(t, _) => t,
            Character(t) => t,
        }
    }

    pub fn inner(&self) -> &Transform {
        match self {
            Player(t, _) => t,
            Character(t) => t,
        }
    }
}

impl AbstractEntity for Entity {
    type Data = GameSharedData;

    fn move_by(&mut self, dp: Point) {
        self.inner_mut().position.add_assign(dp)
    }

    fn move_to(&mut self, p: Point) {
        self.inner_mut().position = p;
    }

    fn get_position(&self) -> Point {
        self.inner().position
    }

    fn set_glyph(&mut self, glyph: Glyph) {
        self.inner_mut().glyph = glyph;
    }

    fn set_glyph_ch(&mut self, ch: char) {
        self.inner_mut().glyph.ch = ch;
    }

    fn set_glyph_fg(&mut self, fg: RGB) {
        self.inner_mut().glyph.fg = fg;
    }

    fn set_glyph_bg(&mut self, bg: RGB) {
        self.inner_mut().glyph.bg = bg;
    }

    fn get_glyph(&self) -> Glyph {
        self.inner().glyph
    }

    fn get_fov(&self) -> HashSet<Point> {
        self.inner().fov.clone()
    }

    fn is_player(&self) -> bool {
        if let Player(_, _) = self { true } else { false }
    }

    fn is_dirty(&self) -> bool {
        match *self {
            Player(_, dirty) => dirty,
            _ => false,
        }
    }

    fn mark_dirty(&mut self) {
        if let Player(_, dirty) = self {
            *dirty = true;
        }
    }

    fn tick(&mut self, data: &Self::Data) {
        match self {
            Player(transform, dirty) if *dirty => *dirty = tick_player(transform, data),
            Character(transform) => tick_character(transform, data),
            _ => {},
        }
    }
}

pub fn tick_player(transform: &mut Transform, data: &GameSharedData) -> bool {
    let old_position = transform.position;
    transform.fov = field_of_view_set(transform.position, 16, &data.map);
    true
}

pub fn tick_character(transform: &mut Transform, data: &GameSharedData) {}