mod pawn;
pub use pawn::{PawnBehaviour, PawnBehaviourState, StorePawnBehaviour};

use zerocopy::transmute;
use crate::shared::PositionF32;

/// Only one beheviour type in this tiny demo
#[derive(Copy, Clone)]
pub enum AnyBehaviour {
    NoBehaviour,
    Pawn(PawnBehaviour),
}

/// The behaviour state is a staging area for the new behaviours. Running behaviours are stored in the world
/// We need a staging area because of the borrow checker. Ex: a behaviour that spawns another behaviour won't
/// be able to store it in the world because it will already be borrowed mutably.
pub struct BehaviourState {
    pub new: Vec<AnyBehaviour>
}

impl BehaviourState {

    pub fn new_behaviour(&mut self, behaviour: impl Into<AnyBehaviour>) {
        self.new.push(behaviour.into());
    }

    pub fn run(data: &mut super::GameData) {
        if !data.behaviours.new.is_empty() {
            Self::run_new_behaviour(data);
        }
       
        Self::run_inner(data);
    }

    fn run_new_behaviour(data: &mut super::GameData) {
        let behaviours: Vec<AnyBehaviour> = data.behaviours.new.drain(..).collect();
        for behaviour in behaviours {
            match behaviour {
                AnyBehaviour::NoBehaviour => {},
                AnyBehaviour::Pawn(pawn) => { pawn::new_behavior(data, pawn);  }
            }
        }
    }

    fn run_inner(data: &mut super::GameData) {
        // In a multithreaded environment, behaviours are run in parallel and distributed through a threadpool
        // alas we're on wasm which is single threaded. 
        pawn::run(data);
    }
}

//
// Store/Load
//

use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};

#[derive(Copy, Clone, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum StoreBehaviourType {
    NoBehaviour { _padding: [u32; 2] },
    PawnIdle { _padding: [u32; 2] },
    PawnMoveToPoint { target: PositionF32 }
}

#[derive(Copy, Clone, Immutable, IntoBytes, TryFromBytes)]
pub struct StoreBehaviour {
    pub entity: [u32; 2],
    pub ty: StoreBehaviourType
}

impl From<AnyBehaviour> for StoreBehaviour {
    fn from(value: AnyBehaviour) -> Self {
        match value {
            AnyBehaviour::NoBehaviour => StoreBehaviour { 
                entity: [0; 2],
                ty: StoreBehaviourType::NoBehaviour { _padding: [0; 2] }
            },
            AnyBehaviour::Pawn(pawn) => StoreBehaviour { 
                entity: transmute!(pawn.entity.to_bits()),
                ty: StoreBehaviourType::from(pawn.ty)
            }
        }
    }
}

impl From<StoreBehaviour> for AnyBehaviour {
    fn from(value: StoreBehaviour) -> Self {
        match value.ty {
            StoreBehaviourType::NoBehaviour {..} => AnyBehaviour::NoBehaviour,
            StoreBehaviourType::PawnIdle {..} | StoreBehaviourType::PawnMoveToPoint {..}  => AnyBehaviour::Pawn(PawnBehaviour::from(value)),
        }
    }
}

impl crate::store::StoreLoad for BehaviourState {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        let stored: Vec<StoreBehaviour> = self.new.drain(..).map(|value| value.into() ).collect();
        writer.write_array(&stored);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let stored: Vec<StoreBehaviour> = unsafe { reader.read_array_transmute().to_vec() };
        let state = BehaviourState {
            new: stored.into_iter().map(|s| s.into() ).collect()
        };

        Ok(state)
    }
}

//
// Other impl
//

impl Default for StoreBehaviour {
    fn default() -> Self {
        StoreBehaviour {
            entity: [0; 2],
            ty: StoreBehaviourType::NoBehaviour { _padding: [0; 2] }
        }
    }
}

impl Default for BehaviourState {
    fn default() -> Self {
        BehaviourState {
            new: Vec::with_capacity(8)
        }
    }
}

impl Default for StoreBehaviourType {
    fn default() -> Self {
        Self::NoBehaviour { _padding: [0; 2] }
    }
}

