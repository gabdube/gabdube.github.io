use crate::GameClient;
use super::common_inputs;

pub fn update(game: &mut GameClient) {
    common_inputs(game);

    let common = &game.data.common;

    if common.primary_mouse_just_pressed() {
        if game.data.gui.position_outside_gui(common.mouse_position) {
            let position = common.mouse_position - common.view_offset;
            game.data.world.clear_selected_sprites();
            game.data.world.select_sprite_at_position(position);
        }
    }

    if common.debug_flags.show_triangle_lookup() {
        if common.debug_flags.show_triangle_lookup_path() {
            highlight_triangle_lookup_path(game);
        } else {
            highlight_hovered_triangle(game);
        }
    }
}

fn highlight_hovered_triangle(game: &mut GameClient) {
    let common = &game.data.common;
    let navigation = &game.data.navigation;
    let debug = &mut game.data.debug;
    let position = common.mouse_position - common.view_offset;
    navigation.debug_triangle_at_position(debug, position);
}

fn highlight_triangle_lookup_path(game: &mut GameClient) {
    let common = &game.data.common;
    let navigation = &game.data.navigation;
    let debug = &mut game.data.debug;
    let position = common.mouse_position - common.view_offset;
    navigation.debug_triangle_lookup_path(debug, position);
}
