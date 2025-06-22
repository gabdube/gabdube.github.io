mod move_to_point;

use hecs::Entity;
use zerocopy::transmute;
use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::data::GameData;
use crate::shared::PositionF32;
use super::{StoreBehaviour, StoreBehaviourType, AnyBehaviour};

#[derive(Copy, Clone)]
pub enum PawnBehaviourType {
    Idle,
    MoveToPoint { target: PositionF32 }
}

#[derive(Copy, Clone)]
pub enum PawnBehaviourStateData {
    None,
}

/// The behaviour state stored in the world
#[derive(Copy, Clone)]
pub struct PawnBehaviourState {
    pub ty: PawnBehaviourType,
    pub data: PawnBehaviourStateData,
    pub step: u32,
}

impl PawnBehaviourState {

    pub fn idle() -> Self {
        PawnBehaviourState { 
            ty: PawnBehaviourType::Idle,
            data: PawnBehaviourStateData::None,
            step: 0
        }
    }

}

/// Behaviour instanced by the app when updating a pawn behaviour
#[derive(Copy, Clone)]
pub struct PawnBehaviour {
    pub entity: Entity,
    pub ty: PawnBehaviourType
}

impl PawnBehaviour {

    pub fn move_to_point(pawn: Entity, target: PositionF32) -> Self {
        PawnBehaviour { entity: pawn, ty: PawnBehaviourType::MoveToPoint { target } }
    }

}

//
// Behaviour updates
//

/// Cancel the current behaviour and replace it by the new one
/// This demo does not have any behaviour with cancelling logic
pub(super) fn new_behavior(data: &mut GameData, behavior: PawnBehaviour) {
    let state = match data.world.pawn_behaviour(behavior.entity) {
        Some(b) => b,
        None => {
            dbg!("Tried to access pawn behaviour on {:?} but no data was returned", behavior.entity);
            return;
        }
    };

    state.ty = behavior.ty;
    state.step = 0;

    match behavior.ty {
        PawnBehaviourType::Idle => { state.data = PawnBehaviourStateData::None; },
        PawnBehaviourType::MoveToPoint { .. } => {  state.data = PawnBehaviourStateData::None; }
    }
}

/// Runs the pawn behaviour
pub(super) fn run(data: &mut GameData) {
    for (entity, behaviour) in data.world.iter_pawn_behaviours().iter() {
        match behaviour.ty {
            PawnBehaviourType::Idle => {},
            PawnBehaviourType::MoveToPoint { .. } => move_to_point::run(entity, behaviour),
        }
    }
}


//
// Store / Load
//

#[derive(Copy, Clone, IntoBytes, TryFromBytes, Immutable)]
pub struct StorePawnBehaviour {
    pub ty: StoreBehaviourType,
    pub step: u32,
}

impl From<PawnBehaviourState> for StorePawnBehaviour {
    fn from(value: PawnBehaviourState) -> Self {
        StorePawnBehaviour {
            ty: StoreBehaviourType::from(value.ty),
            step: value.step,
        }
    }
}

impl From<StorePawnBehaviour> for PawnBehaviourState {
    fn from(value: StorePawnBehaviour) -> Self {
        PawnBehaviourState {
            ty: PawnBehaviourType::from(value.ty),
            data: PawnBehaviourStateData::None,
            step: value.step,
        }
    }
}

impl From<PawnBehaviour> for AnyBehaviour {
    fn from(value: PawnBehaviour) -> Self {
        AnyBehaviour::Pawn(value)
    }
}

impl From<StoreBehaviour> for PawnBehaviour {
    fn from(value: StoreBehaviour) -> Self {
        let entity = Entity::from_bits(transmute!(value.entity)).expect("Corrupted entity data");
        PawnBehaviour { entity, ty: PawnBehaviourType::from(value.ty) }
    }
}

impl From<PawnBehaviourType> for StoreBehaviourType {
    fn from(value: PawnBehaviourType) -> Self {
        match value {
            PawnBehaviourType::Idle => StoreBehaviourType::PawnIdle { _padding: [0; 2] },
            PawnBehaviourType::MoveToPoint { target } => StoreBehaviourType::PawnMoveToPoint { target }
        }
    }
}

impl From<StoreBehaviourType> for PawnBehaviourType {
    fn from(value: StoreBehaviourType) -> Self {
        match value {
            StoreBehaviourType::PawnMoveToPoint { target } => PawnBehaviourType::MoveToPoint { target },
            _ => PawnBehaviourType::Idle,
        }
    }
}
