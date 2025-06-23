use crate::shared::PositionF32;
use crate::GameClient;
use super::common_inputs;

pub fn update(game: &mut GameClient) {
    common_inputs(game);

    let data = &mut game.world_data.data;
    let world = &mut game.world_data.world;
    let common = data.common;

    if common.primary_mouse_just_pressed() {
        if data.gui.position_outside_gui(common.mouse_position) {
            let position = common.mouse_position - common.view_offset;
            world.clear_selected_sprites();
            world.select_sprite_at_position(position);
        }
    }

    if world.selected_sprites().len() > 0 {
        if common.debug_flags.debug_any_path() {
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
    let data = &mut game.world_data.data;
    let nav = &data.navigation;
    let debug = &mut data.debug;
    let common = data.common;

    if let Some(start) = selected_pawn_position(&mut game.world_data.world) {
        let end = common.mouse_position - common.view_offset;

        if common.debug_flags.show_path_rough() {
            nav.debug_rough_path(debug, start, end)
        } else if common.debug_flags.show_path_funnel() {
            nav.debug_funnel(debug, start, end);
        } else {
            nav.debug_path(debug, start, end);
        }
    }
}
