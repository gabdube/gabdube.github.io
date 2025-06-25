use hecs::Entity;
use crate::data::assets::Assets;
use crate::data::world::World;
use crate::data::{GameData, NavigationState};
use crate::shared::{PositionF32, pos};
use super::{PawnBehaviourState, PawnBehaviourStateData, PawnBehaviourType};

pub const STARTUP: u32 = 0;
pub const MOVING: u32 = 1;
pub const DESTINATION_REACHED: u32 = 2;

pub(super) struct MoveToPointParams<'a> {
    navigation: Option<&'a NavigationState>,
    world: &'a World,
    assets: &'a Assets,
    delta: f32,
}

pub(super) fn params<'a>(state: &PawnBehaviourState, world: &'a World, data: &'a GameData) -> MoveToPointParams<'a> {
    match state.step {
        STARTUP => {
            MoveToPointParams {
                navigation: Some(&data.navigation),
                world,
                assets: &data.assets,
                delta: 0.0,
            }
        },
        MOVING => {
            MoveToPointParams {
                navigation: None,
                world,
                assets: &data.assets,
                delta: data.common.time_delta,
            }
        },
        DESTINATION_REACHED | _ => {
            MoveToPointParams {
                navigation: None,
                world,
                assets: &data.assets,
                delta: 0.0
            }
        }
    }
}

pub(super) fn run(
    entity: Entity,
    state: &mut PawnBehaviourState,
    params: MoveToPointParams,
) {
    match state.step {
        STARTUP => startup(entity, state, params),
        MOVING => moving(entity, state, params),
        DESTINATION_REACHED | _ => destination_reached(entity, state, params),
    }
}

fn startup(entity: Entity, state: &mut PawnBehaviourState, params: MoveToPointParams) {
    let nav = unsafe { params.navigation.unwrap_unchecked() };
    let world = params.world;
    let assets = params.assets;
    let end_position = match state.ty {
        PawnBehaviourType::MoveToPoint { target } => target,
        _ => unsafe { ::std::hint::unreachable_unchecked(); }
    };

    let start_position = match world.get_pawn_position(entity) {
        Some(position) => position,
        None => {
            destination_reached(entity, state, params);
            return;
        }
    };

    let mut out_path = Vec::with_capacity(16);
    if !nav.compute_path(start_position, end_position, &mut out_path) {
        destination_reached(entity, state, params);
        return;
    }

    world.set_pawn_animation(entity, assets.atlas.pawn_walk.animate());

    state.data = PawnBehaviourStateData::MoveTo { steps: out_path, current_step: 1 };
    state.step = MOVING;
}

fn moving(entity: Entity, state: &mut PawnBehaviourState, params: MoveToPointParams) {
    let world = params.world;

    let (steps, current_step) = match &mut state.data {
        PawnBehaviourStateData::MoveTo { steps, current_step } => (steps, current_step),
        PawnBehaviourStateData::None => {
            destination_reached(entity, state, params);
            return;
        }
    };

    let position = match world.get_pawn_position(entity) {
        Some(position) => position,
        None => {
            destination_reached(entity, state, params);
            return;
        }
    };

    let mut step_index = *current_step as usize;
    let end_position = steps[step_index];
    let new_position = move_to_with_speed(position, end_position, params.delta, 0.2);
    let dx = (end_position.x - new_position.x) < 0.0;

    world.set_pawn_position(entity, new_position);

    if new_position != end_position {
        world.set_pawn_flipped(entity, dx);
    }

    if new_position == end_position {
        step_index += 1;
        if step_index == steps.len() {
            state.step = DESTINATION_REACHED;
        } else {
            *current_step = step_index as u32;
        }
    }
}

fn destination_reached(entity: Entity, state: &mut PawnBehaviourState, params: MoveToPointParams) {
    let world = params.world;
    let assets = params.assets;
    world.set_pawn_animation(entity, assets.atlas.pawn_idle.animate());

    state.ty = PawnBehaviourType::Idle;
    state.data = PawnBehaviourStateData::None;
    state.step = 0;
}

fn move_to_with_speed(current: PositionF32, target: PositionF32, frame_delta: f32, base_speed: f32) -> PositionF32 {
    let angle = f32::atan2(target.y - current.y, target.x - current.x);
    let speed = base_speed * frame_delta;
    let move_x = speed * f32::cos(angle);
    let move_y = speed * f32::sin(angle);
    let mut updated_position = pos(current.x + move_x, current.y + move_y);

    if f32::abs(updated_position.x - target.x) < 1.0 {
        updated_position.x = target.x;
    }

    if f32::abs(updated_position.y - target.y) < 1.0 {
        updated_position.y = target.y;
    }

    updated_position
}
