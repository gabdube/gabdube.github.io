use crate::shared::PositionF32;
use crate::data::GameWorldData;
use super::{KnightBehaviourType, KnightBehaviourStateData};

pub(super) fn new(world_data: &mut GameWorldData, entity: hecs::Entity, target: PositionF32) {
    let knight_behaviour = world_data.world.knight_behaviour_mut(entity);
    match knight_behaviour.ty {
        KnightBehaviourType::Idle => {
            knight_behaviour.ty = KnightBehaviourType::MoveToPoint { target };
            knight_behaviour.data = KnightBehaviourStateData::MoveTo { steps: Vec::new(), current_step: 0 };
            knight_behaviour.step = 0;
        },
        KnightBehaviourType::MoveToPoint { .. } => {
            // TODO
        }
    }
}
