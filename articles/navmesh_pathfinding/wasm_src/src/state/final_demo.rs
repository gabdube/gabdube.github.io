use crate::data::base::DebugFlags;
use crate::shared::pos;
use crate::GameClient;
use super::shared::{set_insert_sprite, primary_mouse_actions, mouse_moved_actions};
use super::{GameStateValue, GameInputType, common_inputs};


pub fn init(game: &mut GameClient) {
    let wd = &mut game.world_data;

    wd.reset();
    wd.initialize_terrain(20, 16);

    wd.add_castle(pos(253.0, 332.0));

    wd.add_house(pos(606.0, 492.0));
    wd.add_house(pos(343.0, 690.0));
    wd.add_house(pos(82.0, 476.0));
    wd.add_house(pos(179.0, 56.0));
    wd.add_house(pos(602.0, 156.0));

    wd.add_pawn(pos(151.0, 723.0));
    wd.add_pawn(pos(446.0, 128.0));

    wd.compute_navigation();
    
    game.state.value = GameStateValue::Pathfinding;
    wd.data.common.debug_flags.0 = DebugFlags::SHOW_NAVMESH | DebugFlags::SHOW_PATH;
    wd.data.gui.set_debug_flags(wd.data.common.debug_flags);
    wd.data.gui.set_state(game.state.value, GameInputType::Select);

    wd.world.order_sprites(false);
    wd.world.select_sprite_at_position(pos(450.0, 130.0));
}


pub fn update(game: &mut GameClient) {
    common_inputs(game);
    set_insert_sprite(game);

    if game.world_data.data.common.primary_mouse_just_pressed() {
        if game.world_data.data.gui.position_outside_gui(game.world_data.data.common.mouse_position) {
            primary_mouse_actions(game);
        }
    }

    if game.world_data.data.common.mouse_moved() {
        mouse_moved_actions(game);
    }
}
