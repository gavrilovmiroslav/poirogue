use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use bracket_color::prelude::{WHITE, BLACK};
use bracket_lib::prelude::Point;
use rhai::Engine;
use crate::colors::{Color, named_color};

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct Glyph {
    pub ch: u16,
    pub fg: Color,
    pub bg: Color,
}

impl Default for Glyph {
    fn default() -> Self {
        Glyph { ch: ' ' as u16, fg: named_color(WHITE), bg: named_color(BLACK) }
    }
}

impl Glyph {
    pub fn char_only(ch: char) -> Glyph {
        Glyph { ch: ch as u16, fg: named_color(WHITE), bg: named_color(BLACK) }
    }

    pub fn char_fg(ch: char, fg: Color) -> Glyph {
        Glyph { ch: ch as u16, fg, bg: named_color(BLACK) }
    }

    pub fn new(ch: char, fg: Color, bg: Color) -> Glyph {
        Glyph { ch: ch as u16, fg, bg }
    }
}

impl Glyph {
    pub fn set_ch(&mut self, ch: char) {
        self.ch = ch as u16;
    }

    pub fn set_fg(&mut self, fg: Color) {
        self.fg = fg;
    }

    pub fn set_bg(&mut self, bg: Color) {
        self.bg = bg;
    }

    pub fn get_ch(&mut self) -> char {
        (self.ch as u8) as char
    }

    pub fn get_fg(&mut self) -> Color {
        self.fg
    }

    pub fn get_bg(&mut self) -> Color {
        self.bg
    }
}

#[derive(Clone)]
pub struct GlyphMap(i32, pub Vec<Glyph>);

use rhai::plugin::*;        // a "prelude" import for macros

impl GlyphMap {
    pub fn new(w: i32, h: i32) -> GlyphMap {
        let size = (w * h) as usize;
        let mut map = GlyphMap(w, Vec::with_capacity(size));

        for i in 0..size {
            map.1.push(Glyph::char_only(' '));
        }

        map
    }

    pub fn get_at(&self, i: i64, j: i64) -> &Glyph {
        let index = (j * self.0 as i64 + i) as usize;
        self.1.get(index).unwrap()
    }

    pub fn get_ch_at(&self, i: i64, j: i64) -> char {
        let index = (j * self.0 as i64 + i) as usize;
        (self.1.get(index).unwrap().ch as u8) as char
    }

    pub fn get_fg_at(&self, i: i64, j: i64) -> Color {
        let index = (j * self.0 as i64 + i) as usize;
        self.1.get(index).unwrap().fg
    }

    pub fn get_bg_at(&self, i: i64, j: i64) -> Color {
        let index = (j * self.0 as i64 + i) as usize;
        self.1.get(index).unwrap().bg
    }

    pub fn set_ch_at(&mut self, i: i64, j: i64, ch: char) {
        let index = (j * self.0 as i64 + i) as usize;
        let mut glyph = &mut self.1[index];
        glyph.set_ch(ch);
    }

    pub fn set_fg_at(&mut self, i: i64, j: i64, fg: Color) {
        let index = (j * self.0 as i64 + i) as usize;
        let mut glyph = &mut self.1[index];
        glyph.set_fg(fg);
    }

    pub fn set_bg_at(&mut self, i: i64, j: i64, bg: Color) {
        let index = (j * self.0 as i64 + i) as usize;
        let mut glyph = &mut self.1[index];
        glyph.set_bg(bg);
    }

    pub fn set_ch_fg_at(&mut self, i: i64, j: i64, ch: char, fg: Color) {
        let index = (j * self.0 as i64 + i) as usize;
        let mut glyph = &mut self.1[index];
        glyph.set_ch(ch);
        glyph.set_fg(fg);
    }

    pub fn set_ch_fg_bg_at(&mut self, i: i64, j: i64, ch: char, fg: Color, bg: Color) {
        let index = (j * self.0 as i64 + i) as usize;
        let mut glyph = &mut self.1[index];
        glyph.set_ch(ch);
        glyph.set_fg(fg);
        glyph.set_bg(bg);
    }
}

#[export_module]
mod glyphs_mod {
    use crate::colors::Color;
    use crate::glyph::Glyph;

    #[rhai_fn(name="make_glyph")]
    pub fn make_glyph_ch(ch: char) -> Glyph {
        Glyph::char_only(ch)
    }

    #[rhai_fn(name="make_glyph")]
    pub fn make_glyph_ch_fg(ch: char, fg: Color) -> Glyph {
        Glyph::char_fg(ch, fg)
    }

    #[rhai_fn(name="make_glyph")]
    pub fn make_glyph_ch_fg_bg(ch: char, fg: Color, bg: Color) -> Glyph {
        Glyph::new(ch, fg, bg)
    }

    #[rhai_fn(name="get_ch", get="ch")]
    pub fn get_ch(glyph: &mut Glyph) -> char {
        glyph.get_ch()
    }

    #[rhai_fn(name="get_fg", get="fg")]
    pub fn get_fg(glyph: &mut Glyph) -> Color {
        glyph.get_fg()
    }

    #[rhai_fn(name="get_bg", get="bg")]
    pub fn get_bg(glyph: &mut Glyph) -> Color {
        glyph.get_bg()
    }

    #[rhai_fn(name="set_ch", set="ch")]
    pub fn set_ch(glyph: &mut Glyph, ch: char) {
        glyph.set_ch(ch)
    }

    #[rhai_fn(name="set_fg", set="fg")]
    pub fn set_fg(glyph: &mut Glyph, fg: Color) {
        glyph.set_fg(fg)
    }

    #[rhai_fn(name="set_bg", set="bg")]
    pub fn set_bg(glyph: &mut Glyph, bg: Color) {
        glyph.set_bg(bg)
    }
}


pub fn register_glyphs(engine: &mut Engine) {
    engine.register_type::<Glyph>()
        .register_type::<GlyphMap>()
        .register_type::<HashSet<Point>>();

    engine.register_global_module(exported_module!(glyphs_mod).into());
}
