use hecs::Entity;
use crate::data::{CommonParams, NavigationState};
use super::PawnBehaviourState;

pub const STARTUP: u32 = 0;
pub const MOVING: u32 = 1;
pub const DESTINATION_REACHED: u32 = 2;


pub(super) fn run(
    entity: Entity,
    state: &mut PawnBehaviourState,
) {
    match state.step {
        STARTUP => startup(entity),
        MOVING => moving(entity),
        DESTINATION_REACHED | _ => destination_reached(entity),
    }
}

fn startup(entity: Entity) {

}

fn moving(entity: Entity) {

}

fn destination_reached(entity: Entity) {

}

