pub mod game_state;

use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use super::GameClient;

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, TryFromBytes, IntoBytes, Immutable)]
#[repr(u32)]
pub enum GameStateValue {
    #[default]
    Uninitialized,
    Game,
}

#[derive(Default, Copy, Clone)]
pub struct GameState {
    pub value: GameStateValue,
    pub scroll_view: bool,
}

impl crate::store::StoreLoad for GameState {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.value);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut state = GameState::default();
        state.value = reader.try_read()?;
        Ok(state)
    }
}

pub fn init(client: &mut GameClient) {
    client.world_data.reset();
    client.world_data.initialize_terrain(32, 32);
    client.state.value = GameStateValue::Game;
}
