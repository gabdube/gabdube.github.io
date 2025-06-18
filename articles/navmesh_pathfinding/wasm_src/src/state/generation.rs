use crate::GameClient;
use super::shared::{set_insert_sprite, primary_mouse_actions, mouse_moved_actions};
use super::common_inputs;


pub fn update(game: &mut GameClient) {
    common_inputs(game);
    set_insert_sprite(game);

    if game.data.globals.primary_mouse_just_pressed() {
        if game.data.gui.position_outside_gui(game.data.globals.mouse_position) {
            primary_mouse_actions(game);
        }
    }

    if game.data.globals.mouse_moved() {
        mouse_moved_actions(game);
    }
}

