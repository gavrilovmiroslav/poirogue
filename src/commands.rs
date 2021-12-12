
pub enum FlowCommand {
    CycleViews,
    GenerateLevel,
    Exit
}

#[derive(PartialEq)]
pub enum GameFlow {
    Player,
    World,
}

pub enum GameCommand {
    Flow(FlowCommand),
}
