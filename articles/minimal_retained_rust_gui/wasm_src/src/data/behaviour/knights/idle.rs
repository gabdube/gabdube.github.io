use crate::data::GameWorldData;
use super::{KnightBehaviourType, KnightBehaviourStateData};

pub(super) fn new(world_data: &mut GameWorldData, entity: hecs::Entity) {
    let knight_behaviour = world_data.world.knight_behaviour_mut(entity);
    match knight_behaviour.ty {
        KnightBehaviourType::Idle => {},
        KnightBehaviourType::MoveToPoint { .. } => {
            knight_behaviour.ty = KnightBehaviourType::Idle;
            knight_behaviour.data = KnightBehaviourStateData::None;
            knight_behaviour.step = 0;
        }
    }
}
