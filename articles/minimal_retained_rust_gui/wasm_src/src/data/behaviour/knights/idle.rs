use hecs::Entity;
use crate::data::{GameData, World};
use crate::data::sprites::AnimatedSprite;
use super::{KnightBehaviourType, KnightBehaviourState, KnightBehaviourStateData};

pub const INITIALIZE: u32 = 0;
pub const IDLE: u32 = 1;

pub(super) struct IdleParams<'a> {
    idle_animation: &'a AnimatedSprite,
}

pub(super) fn params<'a>(data: &'a GameData) -> IdleParams<'a> {
    IdleParams { idle_animation: &data.assets.atlas.knight_idle }
}

pub(super) fn swap_state(knight_behaviour: &mut KnightBehaviourState) {
    match knight_behaviour.ty {
        KnightBehaviourType::Idle => {},
        KnightBehaviourType::MoveToPoint { .. } => {
            knight_behaviour.ty = KnightBehaviourType::Idle;
            knight_behaviour.data = KnightBehaviourStateData::None;
            knight_behaviour.step = INITIALIZE;
        }
    }
}

pub(super) fn run(entity: Entity, state: &mut KnightBehaviourState, world: &World, params: IdleParams) {
    match state.step {
        INITIALIZE => initialize(entity, state, world, params),
        IDLE => {},
        _ => {}
    }
}

fn initialize(entity: Entity, state: &mut KnightBehaviourState, world: &World, params: IdleParams) {
    world.set_sprite_animation(entity, params.idle_animation.animate());
    state.step = IDLE;
}
