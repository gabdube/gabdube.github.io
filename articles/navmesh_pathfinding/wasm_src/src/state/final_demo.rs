use crate::data::base::DebugFlags;
use crate::shared::pos;
use crate::GameClient;
use super::shared::{set_insert_sprite, primary_mouse_actions, mouse_moved_actions};
use super::{GameStateValue, GameInputType, common_inputs};


pub fn init(game: &mut GameClient) {
    let data = &mut game.data;

    data.reset();
    data.initialize_terrain(20, 16);

    data.add_castle(pos(253.0, 332.0));

    data.add_house(pos(606.0, 492.0));
    data.add_house(pos(343.0, 690.0));
    data.add_house(pos(82.0, 476.0));
    data.add_house(pos(179.0, 56.0));
    data.add_house(pos(602.0, 156.0));

    data.add_pawn(pos(151.0, 723.0));
    data.add_pawn(pos(446.0, 128.0));

    data.compute_navigation();
    
    game.state.value = GameStateValue::Pathfinding;
    game.data.globals.debug_flags.0 |= DebugFlags::SHOW_NAVMESH | DebugFlags::SHOW_PATH_ROUGH;
    game.data.gui.set_debug_flags(game.data.globals.debug_flags);
    game.data.gui.set_state(game.state.value, GameInputType::Select);
}


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
