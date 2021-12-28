use crate::entity::Entity;
use crate::game::GameSharedData;

pub struct Player {

}

impl Default for Player {
    fn default() -> Self {
        Player{}
    }
}

impl Entity<GameSharedData> for Player {
    fn tick(&mut self, data: &GameSharedData) {

    }
}
