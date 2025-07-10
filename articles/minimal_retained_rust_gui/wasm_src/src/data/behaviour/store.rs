use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use zerocopy::transmute;
use crate::shared::PositionF32;
use super::{AnyBehaviour, BehaviourState, KnightBehaviour};

#[derive(Copy, Clone, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum StoreBehaviourType {
    NoBehaviour { _padding: [u32; 2] },
    KnightIdle { _padding: [u32; 2] },
    KnightMoveToPoint { target: PositionF32 },
}

#[derive(Copy, Clone, Immutable, IntoBytes, TryFromBytes)]
pub struct StoreBehaviour {
    pub entity: [u32; 2],
    pub ty: StoreBehaviourType
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


impl From<AnyBehaviour> for StoreBehaviour {
    fn from(value: AnyBehaviour) -> Self {
        match value {
            AnyBehaviour::NoBehaviour => StoreBehaviour { 
                entity: [0; 2],
                ty: StoreBehaviourType::NoBehaviour { _padding: [0; 2] }
            },
            AnyBehaviour::Knight(knight) => StoreBehaviour { 
                entity: transmute!(knight.entity.to_bits()),
                ty: StoreBehaviourType::from(knight.ty)
            }
        }
    }
}

impl From<StoreBehaviour> for AnyBehaviour {
    fn from(value: StoreBehaviour) -> Self {
        match value.ty {
            StoreBehaviourType::NoBehaviour {..} => AnyBehaviour::NoBehaviour,
            StoreBehaviourType::KnightIdle {..} | StoreBehaviourType::KnightMoveToPoint {..}  => AnyBehaviour::Knight(KnightBehaviour::from(value)),
        }
    }
}


impl Default for StoreBehaviour {
    fn default() -> Self {
        StoreBehaviour {
            entity: [0; 2],
            ty: StoreBehaviourType::NoBehaviour { _padding: [0; 2] }
        }
    }
}

impl Default for StoreBehaviourType {
    fn default() -> Self {
        Self::NoBehaviour { _padding: [0; 2] }
    }
}

