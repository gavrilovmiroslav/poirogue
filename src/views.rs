use bracket_lib::prelude::{GREEN, RED, RGB, WHITE};
use crate::rand_gen::get_random_between;
use crate::tiles::{DebugMapTile, MapTile};

pub trait View<D> {
    fn get_description(&self, t: &D) -> String;
    fn get_glyph(&self, t: &D) -> char;
    fn get_color(&self, t: &D) -> RGB;
}

pub fn get_description<D, U: View<D> + ?Sized>(t: &D, u: &U) -> String {
    u.get_description(t)
}

pub fn get_glyph<D, U: View<D> + ?Sized>(t: &D, u: &U) -> char {
    u.get_glyph(t)
}

pub fn get_color<D, U: View<D> + ?Sized>(t: &D, u: &U) -> RGB {
    u.get_color(t)
}