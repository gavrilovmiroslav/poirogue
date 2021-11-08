use crate::BTerm;

pub trait GameSystem {
    fn get_name(&self) -> &str;
    fn init(&mut self);
    fn tick(&mut self, ctx: &mut BTerm);
}