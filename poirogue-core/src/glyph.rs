use serde::{Serialize, Deserialize};
use bracket_color::prelude::{WHITE, BLACK};
use crate::colors::{Color, named_color};

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct Glyph {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

impl Glyph {
    pub fn new(ch: char) -> Glyph {
        Glyph { ch, fg: named_color(WHITE), bg: named_color(BLACK) }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct GlyphOpt {
    pub ch: Option<char>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

impl GlyphOpt {
    pub fn new(ch: char) -> GlyphOpt {
        GlyphOpt { ch: Some(ch), fg: None, bg: None }
    }

    pub fn collect(&self) -> Glyph {
        Glyph {
            ch: self.ch.unwrap_or(' '),
            fg: self.fg.unwrap_or(named_color(WHITE)),
            bg: self.bg.unwrap_or(named_color(BLACK)),
        }
    }
}