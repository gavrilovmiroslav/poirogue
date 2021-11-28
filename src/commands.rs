
pub enum FlowCommand {
    GenerateLevel,
    Exit
}

pub enum GameFlow {
    Player,
    World,
}

pub enum GameCommand {
    Flow(FlowCommand),
}
