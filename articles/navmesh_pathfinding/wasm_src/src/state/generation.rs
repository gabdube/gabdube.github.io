use crate::GameClient;
use super::shared::{set_insert_sprite, primary_mouse_actions, mouse_moved_actions};
use super::common_inputs;


pub fn update(game: &mut GameClient) {
    common_inputs(game);
    set_insert_sprite(game);

    let data = &mut game.world_data.data;
    if data.common.primary_mouse_just_pressed() {
        if data.gui.position_outside_gui(data.common.mouse_position) {
            primary_mouse_actions(game);
        }
    }

    let data = &mut game.world_data.data;
    if data.common.mouse_moved() {
        mouse_moved_actions(game);
    }
}

