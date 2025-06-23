mod move_to_point;

use hecs::Entity;
use zerocopy::transmute;
use crate::data::GameWorldData;
use crate::shared::PositionF32;
use super::{StoreBehaviour, StoreBehaviourType, AnyBehaviour};

#[derive(Copy, Clone)]
pub enum PawnBehaviourType {
    Idle,
    MoveToPoint { target: PositionF32 }
}

pub enum PawnBehaviourStateData {
    None,
    MoveTo { steps: Vec<PositionF32>, current_step: u32 }
}

/// The behaviour state stored in the world
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
pub(super) fn new_behavior(data: &mut GameWorldData, behavior: PawnBehaviour) {
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
pub(super) fn run(world_data: &mut GameWorldData) {
    for (entity, state) in world_data.world.pawn_behaviours().iter() {
        match state.ty {
            PawnBehaviourType::Idle => {},
            PawnBehaviourType::MoveToPoint { .. } => {
                move_to_point::run(entity, state, move_to_point::params(state, &world_data.world, &world_data.data));
            },
        }
    }
}


//
// Store / Load
//

impl crate::store::StoreLoad for PawnBehaviourState {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.step);
        writer.write(&StoreBehaviourType::from(self.ty));
        
        match &self.data {
            PawnBehaviourStateData::None => { 
                writer.write(&0u32);
            },
            PawnBehaviourStateData::MoveTo { steps, current_step } => {
                writer.write(&1u32);
                writer.write(current_step);
                writer.write_array(steps);
            }
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let step = reader.try_read()?;
        let store_ty: StoreBehaviourType = reader.try_read()?;
        let ty = PawnBehaviourType::from(store_ty);

        let data_id: u32 = reader.try_read()?;
        let data = match data_id {
            1 => {
                let current_step = reader.try_read()?;
                let steps = reader.read_array().to_vec();
                PawnBehaviourStateData::MoveTo { steps, current_step }
            }
            _ => PawnBehaviourStateData::None
        };

        Ok(PawnBehaviourState {
            step,
            ty,
            data
        })
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
