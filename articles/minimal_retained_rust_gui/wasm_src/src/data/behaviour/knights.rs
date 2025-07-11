mod idle;
mod move_to;

use hecs::Entity;
use zerocopy::transmute;
use crate::data::{World, GameData};
use crate::shared::PositionF32;
use super::store::{StoreBehaviour, StoreBehaviourType};
use super::AnyBehaviour;


#[derive(Copy, Clone)]
pub enum KnightBehaviourType {
    Idle,
    MoveToPoint { target: PositionF32 }
}

impl KnightBehaviourType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::MoveToPoint { .. } => "MoveToPoint",
        }
    }
}

pub enum KnightBehaviourStateData {
    None,
    MoveTo { steps: Vec<PositionF32>, current_step: u32 }
}

pub struct KnightBehaviourState {
    pub ty: KnightBehaviourType,
    pub data: KnightBehaviourStateData,
    pub step: u32,
}

impl KnightBehaviourState {

    pub fn idle() -> Self {
        KnightBehaviourState { 
            ty: KnightBehaviourType::Idle,
            data: KnightBehaviourStateData::None,
            step: 0
        }
    }

}

/// Behaviour instanced by the app when updating a pawn behaviour
#[derive(Copy, Clone)]
pub struct KnightBehaviour {
    pub entity: Entity,
    pub ty: KnightBehaviourType
}

impl KnightBehaviour {

    pub(super) fn run_all(world: &World, data: &GameData) {
        for (entity, behaviour_state) in world.all_knights_behaviour().iter() {
            match behaviour_state.ty {
                KnightBehaviourType::Idle => idle::run(entity, behaviour_state, world, idle::params(data)),
                KnightBehaviourType::MoveToPoint { .. } => move_to::run(entity, behaviour_state, world, move_to::params(data)),
            }
        }
    }

    pub(super) fn insert_into_world(self, world_data: &mut crate::data::GameWorldData) {
        if !world_data.world.entity_id_mut(self.entity).is_knight() {
            dbg!("KnightBehaviour entity ID is not knight, aborting {:?}", self.ty.name());
            return;
        }

        let state = world_data.world.knight_behaviour_mut(self.entity);

        match self.ty {
            KnightBehaviourType::Idle => idle::swap_state(state),
            KnightBehaviourType::MoveToPoint { target } => move_to::swap_state(state, target)
        }
    }

    pub fn move_to_point(pawn: Entity, target: PositionF32) -> Self {
        KnightBehaviour { entity: pawn, ty: KnightBehaviourType::MoveToPoint { target } }
    }

}

//
// Store / Load
//

impl crate::store::StoreLoad for KnightBehaviourState {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.step);
        writer.write(&StoreBehaviourType::from(self.ty));
        
        match &self.data {
            KnightBehaviourStateData::None => { 
                writer.write(&0u32);
            },
            KnightBehaviourStateData::MoveTo { steps, current_step } => {
                writer.write(&1u32);
                writer.write(current_step);
                writer.write_array(steps);
            }
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let step = reader.try_read()?;
        let store_ty: StoreBehaviourType = reader.try_read()?;
        let ty = KnightBehaviourType::from(store_ty);

        let data_id: u32 = reader.try_read()?;
        let data = match data_id {
            1 => {
                let current_step = reader.try_read()?;
                let steps = reader.read_array().to_vec();
                KnightBehaviourStateData::MoveTo { steps, current_step }
            }
            _ => KnightBehaviourStateData::None
        };

        Ok(KnightBehaviourState {
            step,
            ty,
            data
        })
    }
}

impl From<KnightBehaviour> for AnyBehaviour {
    fn from(value: KnightBehaviour) -> Self {
        AnyBehaviour::Knight(value)
    }
}

impl From<StoreBehaviour> for KnightBehaviour {
    fn from(value: StoreBehaviour) -> Self {
        let entity = Entity::from_bits(transmute!(value.entity)).expect("Corrupted entity data");
        KnightBehaviour { entity, ty: KnightBehaviourType::from(value.ty) }
    }
}

impl From<KnightBehaviourType> for StoreBehaviourType {
    fn from(value: KnightBehaviourType) -> Self {
        match value {
            KnightBehaviourType::Idle => StoreBehaviourType::KnightIdle { _padding: [0; 2] },
            KnightBehaviourType::MoveToPoint { target } => StoreBehaviourType::KnightMoveToPoint { target }
        }
    }
}

impl From<StoreBehaviourType> for KnightBehaviourType {
    fn from(value: StoreBehaviourType) -> Self {
        match value {
            StoreBehaviourType::KnightMoveToPoint { target } => KnightBehaviourType::MoveToPoint { target },
            _ => KnightBehaviourType::Idle,
        }
    }
}
