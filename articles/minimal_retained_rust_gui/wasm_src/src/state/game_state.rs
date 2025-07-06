use crate::shared::{PositionF32, AABB, size};
use crate::GameClient;

pub fn update(game: &mut GameClient) {
    scroll_view_logic(game);
    mouse_click_logic(game);
    mouse_move_logic(game);
}

fn scroll_view_logic(game: &mut GameClient) {
    let common = &mut game.world_data.data.common;

    if common.middle_mouse_just_pressed() {
        game.state.scroll_view = true;
    } else if common.middle_mouse_released() {
        game.state.scroll_view = false;
    } 
    
    if game.state.scroll_view {
        if let Some(delta) = common.mouse_delta() {
            common.view_offset -= delta;
            common.render_flags.set_update_view_offset();
        }
    }
}

fn mouse_click_logic(game: &mut GameClient) {
    let common = &game.world_data.data.common;

    if common.primary_mouse_just_pressed() {
        game.state.selection_start = Some(game.world_data.game_mouse_position());
    }

    if common.primary_mouse_just_released() {
        if let Some(start) = game.state.selection_start.take() {
            let selection = selection_rect(start, game.world_data.game_mouse_position());
            if selection.size() <= size(1.0, 1.0) {
                game.world_data.world.clear_selected_sprites();
                game.world_data.world.select_sprite_at_position(start);
            } else {

            }
        }
    }
}

fn mouse_move_logic(game: &mut GameClient) {
    if let Some(start) = game.state.selection_start {
        let selection = selection_rect(start, game.world_data.game_mouse_position());
        if selection.size() > size(1.0, 1.0) {
            game.world_data.data.debug.draw_rect(selection, 2.0, [255, 255, 255, 255]);
        }
    }
}

fn selection_rect(mut start: PositionF32, mut stop: PositionF32) -> AABB {
    if start.x > stop.x { ::std::mem::swap(&mut start.x, &mut stop.x); }
    if start.y > stop.y { ::std::mem::swap(&mut start.y, &mut stop.y); }
    AABB { left: start.x, top: start.y, right: stop.x, bottom: stop.y }
}
