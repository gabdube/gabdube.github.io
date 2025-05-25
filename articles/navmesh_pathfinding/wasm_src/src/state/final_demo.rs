use crate::shared::pos;
use crate::GameClient;
use super::GameState;

pub fn init(game: &mut GameClient) {
    game.data.add_pawn(pos(100.0, 100.0));
    game.state = GameState::FinalDemo;
}

pub fn update(_game: &mut GameClient, _time: f64) {
    
}
