use crate::data::base::StaticSprite;

use crate::shared::{SizeF32, PositionF32, pos};
use crate::GameClient;
use super::GameInputType;

pub(super) fn set_insert_sprite(game: &mut GameClient) {
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

pub(super) fn set_insert_sprite_value(game: &mut GameClient, sprite: StaticSprite) {
    let position = game.data.common.mouse_position;
    if game.data.gui.position_outside_gui(position) {
        game.data.world.set_insert_sprite(center_sprite(position, sprite.texcoord.size()), sprite);
    } else {
        game.data.world.clear_insert_sprite();
    }
}

pub(super) fn primary_mouse_actions(game: &mut GameClient) {
    let common = &game.data.common;
    let position = common.mouse_position - common.view_offset;
    match game.state.input_type {
        GameInputType::PlaceCastle => {
            if position_inside_terrain(&game.data.terrain, position) {
                let sprite = game.data.assets.atlas.castle;
                game.data.add_castle(center_sprite(position, sprite.texcoord.size()));
                game.data.compute_navigation();
            }
        },
        GameInputType::PlaceHouse => {
            if position_inside_terrain(&game.data.terrain, position) {
                let sprite = game.data.assets.atlas.house;
                game.data.add_house(center_sprite(position, sprite.texcoord.size()));
                game.data.compute_navigation();
            }
        },
        GameInputType::PlacePawn => {
            if position_inside_terrain(&game.data.terrain, position) {
                let sprite = game.data.assets.atlas.pawn_idle.sprite();
                game.data.add_pawn(center_sprite(position, sprite.texcoord.size()));
            }
        }
        GameInputType::Delete => {
            game.data.delete_sprite_at_position(position);
            game.data.compute_navigation();
        },
        GameInputType::Select => {
            game.data.world.clear_selected_sprites();
            game.data.world.select_sprite_at_position(position);
        }
    }
}

pub(super) fn secondary_mouse_actions(game: &mut GameClient) {
    let common = game.data.common;
    if game.data.gui.position_outside_gui(common.mouse_position) {
        let world = &mut game.data.world;
        let selected_pawn = world.selected_sprites().first().copied();
        if let Some(selected_pawn) = selected_pawn {
            let position = common.mouse_position - common.view_offset;
            game.data.behaviours.new_behaviour(crate::data::behaviour::PawnBehaviour::move_to_point(selected_pawn, position));
        }
    }
}

pub(super) fn mouse_moved_actions(game: &mut GameClient) {
    match game.state.input_type {
        GameInputType::Delete => {
            let common = &game.data.common;
            let position = common.mouse_position - common.view_offset;
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

pub(super) fn center_sprite(position: PositionF32, size: SizeF32) -> PositionF32 {
    pos(position.x - (size.width * 0.5), position.y - size.height)
}

fn position_inside_terrain(terrain: &crate::data::terrain::Terrain, position: PositionF32) -> bool {
    terrain.rect().point_inside(position)
}
