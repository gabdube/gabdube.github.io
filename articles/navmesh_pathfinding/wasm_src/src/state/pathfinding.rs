use crate::GameClient;
use super::common_inputs;

pub fn update(game: &mut GameClient) {
    common_inputs(game);

    let globals = &game.data.globals;

    if globals.primary_mouse_just_pressed() {
        if game.data.gui.position_outside_gui(globals.mouse_position) {
            let position = globals.mouse_position - globals.view_offset;
            game.data.world.clear_selected_sprites();
            game.data.world.select_sprite_at_position(position);
        }
    }

}
