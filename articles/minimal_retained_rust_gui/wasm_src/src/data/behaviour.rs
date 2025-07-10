mod knights;
pub use knights::{KnightBehaviour, KnightBehaviourState};

mod store;

#[derive(Copy, Clone)]
pub enum AnyBehaviour {
    NoBehaviour,
    Knight(KnightBehaviour),
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

    pub fn run(world_data: &mut super::GameWorldData) {
        if !world_data.data.behaviours.new.is_empty() {
            Self::run_new_behaviour(world_data);
        }
       
        Self::run_inner(world_data);
    }

    fn run_new_behaviour(world_data: &mut super::GameWorldData) {
        let behaviours: Vec<AnyBehaviour> = world_data.data.behaviours.new.drain(..).collect();
        for behaviour in behaviours {
            match behaviour {
                AnyBehaviour::NoBehaviour => {},
                AnyBehaviour::Knight(knight_behaviour) => { knight_behaviour.insert_into_world(world_data); }
            }
        }
    }

    fn run_inner(world_data: &mut super::GameWorldData) {

    }
}


impl Default for BehaviourState {
    fn default() -> Self {
        BehaviourState {
            new: Vec::with_capacity(8)
        }
    }
}
