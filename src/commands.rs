use serde::{Serialize, Deserialize};

#[derive(PartialEq, Serialize, Deserialize)]
pub enum FlowCommand {
    CycleViews,
    GenerateLevel,
    ReloadViewConfigs,
    Exit
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum HackCommand {
    UnlockAllDoors,
    LockAllDoors
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum GameFlow {
    Player,
    World,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum GameCommand {
    Flow(FlowCommand),
    Hack(HackCommand),
}
