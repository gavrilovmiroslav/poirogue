use serde::{Serialize, Deserialize};

#[derive(PartialEq, Serialize, Deserialize)]
pub enum FlowCommand {
    CycleViews,
    GenerateLevel,
    Exit
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum GameFlow {
    Player,
    World,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum GameCommand {
    Flow(FlowCommand),
}
