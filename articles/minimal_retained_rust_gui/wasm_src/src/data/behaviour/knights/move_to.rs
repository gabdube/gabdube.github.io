use hecs::Entity;
use crate::shared::PositionF32;
use crate::data::behaviour::shared::move_to_with_speed;
use crate::data::{GameData, World, Assets, NavigationState};
use super::{KnightBehaviourType, KnightBehaviourState, KnightBehaviourStateData};

pub const INITIALIZE: u32 = 0;
pub const COMPUTE_PATH: u32 = 1;
pub const MOVING: u32 = 2;

pub(super) struct MoveToParams<'a> {
    pub assets: &'a Assets,
    pub nav: &'a NavigationState,
    pub delta: f32
}

pub(super) fn params<'a>(data: &'a GameData) -> MoveToParams<'a> {
    MoveToParams { assets: &data.assets, nav: &data.navigation, delta: data.common.time_delta }
}

pub(super) fn swap_state(knight_behaviour: &mut KnightBehaviourState, target: PositionF32) {
    match knight_behaviour.ty {
        KnightBehaviourType::Idle => {
            knight_behaviour.step = INITIALIZE;
            knight_behaviour.data = KnightBehaviourStateData::MoveTo { steps: Vec::new(), current_step: 0 };
        },
        KnightBehaviourType::MoveToPoint { .. } => {
            if knight_behaviour.step > INITIALIZE {
                knight_behaviour.step = COMPUTE_PATH;
            } else {
                knight_behaviour.step = INITIALIZE;
            }
        }
    }

    knight_behaviour.ty = KnightBehaviourType::MoveToPoint { target };
}

pub(super) fn run(entity: Entity, state: &mut KnightBehaviourState, world: &World, params: MoveToParams) {
    match state.step {
        INITIALIZE => initialize(entity, state, world, params),
        COMPUTE_PATH => compute_path(entity, state, world, params),
        MOVING => moving(entity, state, world, params),
        _ => { destination_reached(state); }
    }
}

fn initialize(entity: Entity, state: &mut KnightBehaviourState, world: &World, params: MoveToParams) {
    let animation = params.assets.atlas.knight_run.animate();
    world.set_sprite_animation(entity, animation);
    state.step = COMPUTE_PATH;
}

fn compute_path(entity: Entity, state: &mut KnightBehaviourState, world: &World, params: MoveToParams) {
    let target_position = match state.ty {
        KnightBehaviourType::MoveToPoint { target } => target,
        _ => unsafe { ::std::hint::unreachable_unchecked() } // Can only be called if state.ty is MoveToPoint
    };

    let (steps, current_step) = match &mut state.data {
        KnightBehaviourStateData::MoveTo { steps, current_step } => (steps, current_step),
        _ => unsafe { ::std::hint::unreachable_unchecked() } // Can only be called if state.ty is MoveToPoint
    };

    let start_position = match world.sprite_base_position(entity) {
        Some(position) => position,
        None => unsafe { ::std::hint::unreachable_unchecked() } // Entity must be a knight
    };

    // Try to reuse steps memory
    steps.clear();

    if !params.nav.compute_path(start_position, target_position, steps) {
        destination_reached(state);
        return;
    }

    // Steps 0 is the starting position
    *current_step = 1;

    state.step = MOVING;
}

fn moving(entity: Entity, state: &mut KnightBehaviourState, world: &World, params: MoveToParams) {
    let (steps, current_step) = match &mut state.data {
        KnightBehaviourStateData::MoveTo { steps, current_step } => (steps, current_step),
        _ => unsafe { ::std::hint::unreachable_unchecked() } // Can only be called if state.ty is MoveToPoint
    };

    let position = match world.sprite_base_position(entity) {
        Some(position) => position,
        None => unsafe { ::std::hint::unreachable_unchecked() } // Entity must be a knight
    };

    let mut step_index = *current_step as usize;
    let end_position = steps[step_index];
    let new_position = move_to_with_speed(position, end_position, params.delta, 0.2);
    let dx = (end_position.x - new_position.x) < 0.0;

    if new_position.roughly_equal(end_position) {
        step_index += 1;
        if step_index == steps.len() {
            world.set_sprite_position(entity, new_position, true);
            destination_reached(state);
        } else {
            *current_step = step_index as u32;
        }
    } else {
        world.flip_sprite(entity, dx);
        world.set_sprite_position(entity, new_position, false);
    }
}

fn destination_reached(state: &mut KnightBehaviourState) {
    super::idle::swap_state(state);
}

