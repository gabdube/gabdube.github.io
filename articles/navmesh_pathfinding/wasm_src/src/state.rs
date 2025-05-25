pub mod uninitialized;
pub mod final_demo;

use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};

#[derive(TryFromBytes, IntoBytes, Immutable)]
#[repr(u32)]
pub enum GameState {
    Uninitialized,
    FinalDemo
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Uninitialized
    }
}
