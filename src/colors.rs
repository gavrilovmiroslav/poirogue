use bracket_color::prelude::{RGB, HSV};

pub type Color = HSV;

pub trait ColorShifter {
    fn hue_shift(&self, sh: f32) -> Self;
    fn darken(&self, sh: f32) -> Self;
    fn lighten(&self, sh: f32) -> Self;
    fn desaturate(&self, sh: f32) -> Self;
}

impl ColorShifter for Color {
    fn hue_shift(&self, sh: f32) -> Self {
        let mut hsv = self.clone();
        hsv.h += sh;
        hsv
    }

    fn darken(&self, sh: f32) -> Self {
        let mut hsv = self.clone();
        hsv.v -= sh;
        if hsv.v < 0.0 { hsv.v = 0.0 }
        hsv
    }

    fn lighten(&self, sh: f32) -> Self {
        let mut hsv = self.clone();
        hsv.v += sh;
        if hsv.v > 1.0 { hsv.v = 1.0 }
        hsv
    }

    fn desaturate(&self, sh: f32) -> Self {
        let mut hsv = self.clone();
        hsv.s += sh;
        if hsv.s > 1.0 { hsv.s = 1.0 }
        if hsv.s < 0.0 { hsv.s = 0.0 }
        hsv
    }
}

pub fn named_color(name: (u8, u8, u8)) -> Color {
    RGB::named(name).to_hsv()
}