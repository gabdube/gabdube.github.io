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

    if globals.debug_flags.show_triangle_lookup() {
        if globals.debug_flags.show_triangle_lookup_path() {
            highlight_triangle_lookup_path(game);
        } else {
            highlight_hovered_triangle(game);
        }
    }
}

fn highlight_hovered_triangle(game: &mut GameClient) {
    let globals = &game.data.globals;
    let navigation = &game.data.navigation;
    let debug = &mut game.data.debug;
    let position = globals.mouse_position - globals.view_offset;
    navigation.debug_triangle_at_position(debug, position);
}

fn highlight_triangle_lookup_path(game: &mut GameClient) {
    let globals = &game.data.globals;
    let navigation = &game.data.navigation;
    let debug = &mut game.data.debug;
    let position = globals.mouse_position - globals.view_offset;
    navigation.debug_triangle_lookup_path(debug, position);
}
