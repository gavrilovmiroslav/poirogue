
pub trait CommandQueue {
    type CommandType: Sync + Send;

    fn get_next(&mut self) -> Option<Self::CommandType>;
}
