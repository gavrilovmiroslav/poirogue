use crate::{Point, RGB};

pub struct Renderable {
    pub glyph: char,
    pub position: Point,
    pub color: RGB,
}
