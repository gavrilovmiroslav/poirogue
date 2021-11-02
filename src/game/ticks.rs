use serde::{Serialize, Deserialize};
use crate::game_state::{TickStates};

#[derive(Serialize, Deserialize)]
pub enum ActionType {
    Use, Drop, Target, Help, Equip, Unequip
}

#[derive(Serialize, Deserialize)]
pub enum TurnType {
    PlayerTurn,
    WorldTurn, Submenu(ActionType)
}

#[derive(Serialize, Deserialize)]
pub enum TickType {
    None, MainMenu, InGame(TurnType), GameOver
}

impl TickStates for TickType {}

impl Default for TickType {
    fn default() -> TickType {
        TickType::None
    }
}