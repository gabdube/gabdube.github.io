use crate::shared::pos;
use crate::GameClient;
use super::GameState;

pub fn init(game: &mut GameClient) {
    game.data.reset();
    game.data.initialize_terrain(32, 16);
    game.data.add_pawn(pos(100.0, 100.0));
    game.state = GameState::FinalDemo;
}

pub fn update(game: &mut GameClient, _time: f64) {
    
    // for (_, sprite) in game.data.world.iter_all_sprites() {
    //     game.data.debug.draw_rect(sprite.rect(), 2.0, [255, 0, 0, 255]);
    // }

}
