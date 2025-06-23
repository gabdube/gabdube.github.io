//! Behaviour helper function for the world
//! Actual behaviour logic is located in `src\data\behaviour.rs`
use hecs::{Entity, QueryBorrow};
use crate::data::behaviour::PawnBehaviourState;
use super::World;

impl World {
    pub fn pawn_behaviour(&mut self, pawn: Entity) -> Option<&mut PawnBehaviourState> {
        self.inner.query_one_mut::<&mut PawnBehaviourState>(pawn).ok()
    }

    pub fn pawn_behaviours(&self) -> QueryBorrow<&mut PawnBehaviourState> {
        self.inner.query::<&mut PawnBehaviourState>()
    }
}
