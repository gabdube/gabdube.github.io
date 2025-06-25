use crate::data::base::StaticSprite;

use crate::shared::{SizeF32, PositionF32, pos};
use crate::GameClient;
use super::GameInputType;

pub(super) fn set_insert_sprite(game: &mut GameClient) {
    let assets = &game.world_data.data.assets;
    let world = &mut game.world_data.world;

    match game.state.input_type {
        GameInputType::PlaceCastle => {
            set_insert_sprite_value(game, assets.atlas.castle);
        },
        GameInputType::PlaceHouse => {
            set_insert_sprite_value(game, assets.atlas.house);
        },
        GameInputType::PlacePawn => {
            let sprite = assets.atlas.pawn_idle.sprite();
            set_insert_sprite_value(game, sprite);
        }
        GameInputType::Delete => {
            world.clear_insert_sprite();
        },
        GameInputType::Select => {
            world.clear_insert_sprite();
        }
    }
}

pub(super) fn set_insert_sprite_value(game: &mut GameClient, sprite: StaticSprite) {
    let data = &game.world_data.data;
    let world = &mut game.world_data.world;

    let position = data.common.mouse_position;
    if data.gui.position_outside_gui(position) {
        world.set_insert_sprite(center_sprite(position, sprite.texcoord.size()), sprite);
    } else {
        world.clear_insert_sprite();
    }
}

pub(super) fn primary_mouse_actions(game: &mut GameClient) {
    let world_data = &mut game.world_data;
    let data = &world_data.data;
    let world = &mut world_data.world;

    let common = &data.common;
    let position = common.mouse_position - common.view_offset;

    match game.state.input_type {
        GameInputType::PlaceCastle => {
            if position_inside_terrain(&data.terrain, position) {
                let sprite = data.assets.atlas.castle;
                world_data.add_castle(center_sprite(position, sprite.texcoord.size()));
                world_data.compute_navigation();
            }
        },
        GameInputType::PlaceHouse => {
            if position_inside_terrain(&data.terrain, position) {
                let sprite = data.assets.atlas.house;
                world_data.add_house(center_sprite(position, sprite.texcoord.size()));
                world_data.compute_navigation();
            }
        },
        GameInputType::PlacePawn => {
            dbg!("TEST");
            if position_inside_terrain(&data.terrain, position) {
                let sprite = data.assets.atlas.pawn_idle.sprite();
                world_data.add_pawn(center_sprite(position, sprite.texcoord.size()));
            }
        }
        GameInputType::Delete => {
            world_data.delete_sprite_at_position(position);
            world_data.compute_navigation();
        },
        GameInputType::Select => {
            world.clear_selected_sprites();
            world.select_sprite_at_position(position);
        }
    }
}

pub(super) fn secondary_mouse_actions(game: &mut GameClient) {
    use crate::data::behaviour::PawnBehaviour;
    
    let data = &mut game.world_data.data;
    let world = &mut game.world_data.world;
    let common = data.common;

    if data.gui.position_outside_gui(common.mouse_position) {
        let selected = world.selected_sprites().first().copied();
        if let Some(selected) = selected {
            if world.is_pawn(selected) {
                let position = common.mouse_position - common.view_offset;
                data.behaviours.new_behaviour(PawnBehaviour::move_to_point(selected, position));
            }
        }
    }
}

pub(super) fn mouse_moved_actions(game: &mut GameClient) {
    let world = &mut game.world_data.world;
    let data = &mut game.world_data.data;
    let common = &data.common;

    match game.state.input_type {
        GameInputType::Delete => {
            let position = common.mouse_position - common.view_offset;
            let hovered_new = world.sprite_at_position(position);
            let hovered_old = game.state.hovered_entity;
            if hovered_new != hovered_old {
                if let Some(old) = hovered_old {
                    world.clear_sprite_highlight(old);
                }
                if let Some(new) = hovered_new {
                    world.set_sprite_highlight(new, [255, 0, 0]);
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
