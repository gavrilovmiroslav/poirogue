use std::collections::HashSet;
use bracket_lib::prelude::Point;
use serde::{Serialize, Deserialize};
//use crate::entity::Entity;

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum FlowCommand {
    CycleViews,
    GenerateLevel,
    ReloadViewConfigs,
    Exit
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum HackCommand {
    UnlockAllDoors,
    LockAllDoors
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Eq)]
pub enum GameplayContext {
    MainGame,
    Inventory,
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum ActionCommand {
    MoveBy(i32, i32),
    MoveTo(i32, i32),
    FovChange(Vec<(i32, i32)>),
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum GameCommand {
    Flow(FlowCommand),
    Act(ActionCommand),
    Hack(HackCommand),
}
