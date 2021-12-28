
pub trait Entity<Data> {
    fn tick(&mut self, data: &Data);
}