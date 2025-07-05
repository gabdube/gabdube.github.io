use crate::GameClient;

pub fn update(game: &mut GameClient) {
    scroll_view_logic(game);
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
