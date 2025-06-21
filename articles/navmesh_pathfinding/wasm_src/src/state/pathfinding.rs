use crate::shared::PositionF32;
use crate::GameClient;
use super::common_inputs;

pub fn update(game: &mut GameClient) {
    common_inputs(game);

    let globals = game.data.globals;

    if globals.primary_mouse_just_pressed() {
        if game.data.gui.position_outside_gui(globals.mouse_position) {
            let position = globals.mouse_position - globals.view_offset;
            game.data.world.clear_selected_sprites();
            game.data.world.select_sprite_at_position(position);
        }
    }

    if game.data.world.selected_sprites().len() > 0 {
        if globals.debug_flags.debug_any_path() {
            debug_pathfinding(game);
        }
    }
}

fn selected_pawn_position(world: &mut crate::data::world::World) -> Option<PositionF32> {
    // Only one item can be selected at a time in this demo
    world.selected_sprites().first().copied()
        .and_then(|entity| world.get_pawn(entity) )
        .map(|sprite| sprite.base_position() )
}

fn debug_pathfinding(game: &mut GameClient) {
    let globals = game.data.globals;
    let nav = &game.data.navigation;
    let debug = &mut game.data.debug;

    if let Some(start) = selected_pawn_position(&mut game.data.world) {
        let end = globals.mouse_position - globals.view_offset;

        if globals.debug_flags.show_path_rough() {
            nav.debug_rough_path(debug, start, end)
        } else if globals.debug_flags.show_path_funnel() {
            if globals.secondary_mouse_just_pressed() {
                nav.debug_path(debug, start, end);
            }
            nav.debug_funnel(debug, start, end);
        } else {
            nav.debug_path(debug, start, end);
        }
    }
}
