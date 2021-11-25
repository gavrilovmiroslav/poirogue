use bracket_lib::prelude::{Point, RGB};

pub struct Glyph {
    pub ch: char,
    pub fg: RGB,
    pub bg: RGB,
}

impl Glyph {
    pub fn new(ch: char, fg: RGB, bg: RGB) -> Glyph {
        Glyph { ch, fg, bg }
    }
}

pub enum Dist {
    By(Point),
    To(Point)
}
