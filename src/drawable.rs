use crate::{Point, RGB};

pub struct Drawable {
    pub glyph: char,
    pub position: Point,
    pub color: RGB,
}

impl Drawable {
    pub fn new(c: char, p: Point, rgb: RGB) -> Drawable {
        Drawable { glyph: c, position: p, color: rgb }
    }
}
