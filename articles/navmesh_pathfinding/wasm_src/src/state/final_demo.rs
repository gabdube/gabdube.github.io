use crate::shared::pos;
use crate::GameClient;
use super::{GameStateValue, GameInputType, common_inputs};

pub fn init(game: &mut GameClient) {
    game.data.reset();
    game.data.initialize_terrain(32, 16);
    game.data.add_pawn(pos(100.0, 100.0));
    game.state.value = GameStateValue::FinalDemo;
    game.data.gui.set_state(game.state.value, GameInputType::Select);
}

pub fn update(game: &mut GameClient) {
    common_inputs(game);
}
