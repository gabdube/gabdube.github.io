pub mod running;

use running::RunningState;

pub enum GameState {
    Uninitialized,
    Running(RunningState),
}

impl GameState {
    pub fn running<'a>(&'a mut self) -> &'a mut RunningState {
        match self {
            GameState::Running(state) => state,
            _ => panic!("State is not running!")
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Uninitialized
    }
}

impl crate::store::StoreLoad for GameState {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        match self {
            GameState::Uninitialized => {
                writer.write(&1u32);
            },
            GameState::Running(state) => {
                writer.write(&2u32);
                state.store(writer);
            }
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let id: u32 = reader.try_read()?;
        let state = match id {
            1 => Ok(GameState::Uninitialized),
            2 => Ok(GameState::Running(RunningState::load(reader)?)),
            value => { Err(assets_err!("Unknown identifier for GameState: {value}")) }
        };

       state
    }
}

