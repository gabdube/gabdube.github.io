use crate::data::base::StaticSprite;
use crate::shared::{SizeF32, PositionF32, pos};
use crate::GameClient;
use super::{GameStateValue, GameInputType, common_inputs};

// for (_, sprite) in game.data.world.iter_all_sprites() {
//     game.data.debug.draw_rect(sprite.rect(), 2.0, [255, 0, 0, 255]);
// }


pub fn init(game: &mut GameClient) {
    game.data.reset();
    game.data.initialize_terrain(18, 16);

    game.data.add_castle(pos(253.0, 332.0));

    game.data.add_house(pos(606.0, 492.0));
    game.data.add_house(pos(343.0, 690.0));
    game.data.add_house(pos(82.0, 476.0));
    game.data.add_house(pos(179.0, 56.0));
    game.data.add_house(pos(602.0, 156.0));

    game.data.add_pawn(pos(151.0, 723.0));
    game.data.add_pawn(pos(446.0, 128.0));
    
    game.state.value = GameStateValue::Generation;
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

fn set_insert_sprite(game: &mut GameClient) {
    match game.state.input_type {
        GameInputType::PlaceCastle => {
            set_insert_sprite_value(game, game.data.assets.atlas.castle);
        },
        GameInputType::PlaceHouse => {
            set_insert_sprite_value(game, game.data.assets.atlas.house);
        },
        GameInputType::PlacePawn => {
            let sprite = game.data.assets.atlas.pawn_idle.sprite();
            set_insert_sprite_value(game, sprite);
        }
        GameInputType::Delete => {
            game.data.world.clear_insert_sprite();
        },
        GameInputType::Select => {
            game.data.world.clear_insert_sprite();
        }
    }
}

fn set_insert_sprite_value(game: &mut GameClient, sprite: StaticSprite) {
    let position = game.data.globals.mouse_position;
    if game.data.gui.position_outside_gui(position) {
        game.data.world.set_insert_sprite(center_sprite(position, sprite.texcoord.size()), sprite);
    } else {
        game.data.world.clear_insert_sprite();
    }
}

fn primary_mouse_actions(game: &mut GameClient) {
    let globals = &game.data.globals;
    let position = globals.mouse_position - globals.view_offset;
    match game.state.input_type {
        GameInputType::PlaceCastle => {
            let sprite = game.data.assets.atlas.castle;
            game.data.add_castle(center_sprite(position, sprite.texcoord.size()));
        },
        GameInputType::PlaceHouse => {
            let sprite = game.data.assets.atlas.house;
            game.data.add_house(center_sprite(position, sprite.texcoord.size()));
        },
        GameInputType::PlacePawn => {
            let sprite = game.data.assets.atlas.pawn_idle.sprite();
            game.data.add_pawn(center_sprite(position, sprite.texcoord.size()));
        }
        GameInputType::Delete => {
            game.data.world.delete_sprite_at_position(position);
        },
        GameInputType::Select => {
            game.data.world.clear_selected_sprites();
            game.data.world.select_sprite_at_position(position);
        }
    }
}

fn mouse_moved_actions(game: &mut GameClient) {
    match game.state.input_type {
        GameInputType::Delete => {
            let globals = &game.data.globals;
            let position = globals.mouse_position - globals.view_offset;
            let hovered_new = game.data.world.sprite_at_position(position);
            let hovered_old = game.state.hovered_entity;
            if hovered_new != hovered_old {
                if let Some(old) = hovered_old {
                    game.data.world.clear_sprite_highlight(old);
                }
                if let Some(new) = hovered_new {
                    game.data.world.set_sprite_highlight(new, [255, 0, 0]);
                }
                game.state.hovered_entity = hovered_new;
            }
        },
        _ => {}
    }
}

fn center_sprite(position: PositionF32, size: SizeF32) -> PositionF32 {
    pos(position.x - (size.width * 0.5), position.y - size.height)
}
