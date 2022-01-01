use serde::{Serialize, Deserialize};
use bracket_color::prelude::{WHITE, BLACK, RGB};

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct Glyph {
    pub ch: char,
    pub fg: RGB,
    pub bg: RGB,
}

impl Glyph {
    pub fn new(ch: char) -> Glyph {
        Glyph { ch, fg: RGB::named(WHITE), bg: RGB::named(BLACK) }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct GlyphOpt {
    pub ch: Option<char>,
    pub fg: Option<RGB>,
    pub bg: Option<RGB>,
}

impl GlyphOpt {
    pub fn new(ch: char) -> GlyphOpt {
        GlyphOpt { ch: Some(ch), fg: None, bg: None }
    }

    pub fn collect(&self) -> Glyph {
        Glyph {
            ch: self.ch.unwrap_or(' '),
            fg: self.fg.unwrap_or(RGB::named(WHITE)),
            bg: self.bg.unwrap_or(RGB::named(BLACK)),
        }
    }
}