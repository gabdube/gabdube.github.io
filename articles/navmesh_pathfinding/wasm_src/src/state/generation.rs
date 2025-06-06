use crate::data::base::StaticSprite;
use crate::shared::{SizeF32, PositionF32, pos};
use crate::GameClient;
use super::{GameStateValue, GameInputType, common_inputs};

  //game.data.add_pawn(crate::shared::pos(100.0, 100.0));
    // for (_, sprite) in game.data.world.iter_all_sprites() {
    //     game.data.debug.draw_rect(sprite.rect(), 2.0, [255, 0, 0, 255]);
    // }


pub fn init(game: &mut GameClient) {
    game.data.reset();
    game.data.initialize_terrain(32, 16);
    
    game.state.value = GameStateValue::Generation;
    game.data.gui.set_state(game.state.value, GameInputType::Select);
}

pub fn update(game: &mut GameClient) {
    common_inputs(game);
    set_insert_sprite(game);

    if game.data.globals.primary_mouse_just_pressed() {
        primary_mouse_action(game);
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

fn primary_mouse_action(game: &mut GameClient) {
    let globals = &game.data.globals;
    if game.data.gui.position_inside_gui(globals.mouse_position) {
        return;
    }

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
            if let Some(sprite) = game.data.world.sprite_at_position(position) {
            }
        },
        GameInputType::Select => {
            if let Some(sprite) = game.data.world.sprite_at_position(position) {
                dbg!("{:?}", sprite.rect().splat())
            }
        }
    }
}

fn center_sprite(position: PositionF32, size: SizeF32) -> PositionF32 {
    pos(position.x - (size.width * 0.5), position.y - size.height)
}
