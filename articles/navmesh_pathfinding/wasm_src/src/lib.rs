#[macro_use]
mod logging;

#[macro_use]
mod error;


mod shared;
mod data;
mod state;
mod output;
mod store;

use fnv::FnvHashMap;
use error::Error;
use store::StoreLoad;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct GameClientInit {
    pub(crate) assets_bundle: String,
    pub(crate) text_assets: FnvHashMap<String, String>
}

#[wasm_bindgen]
impl GameClientInit {

    pub fn new() -> Self {
        GameClientInit {
            assets_bundle: String::new(),
            text_assets: FnvHashMap::default(),
        }
    }

    pub fn set_assets_bundle(&mut self, text: String) {
        self.assets_bundle = text;
    }

    pub fn upload_text_asset(&mut self, name: String, value: String) {
        self.text_assets.insert(name, value);
    }


}


/// The game data and the game state
#[wasm_bindgen]
#[derive(Default)]
pub struct GameClient {
    data: data::GameData,
    state: state::GameState,
    output: output::GameOutput,
}

#[wasm_bindgen]
impl GameClient {
    pub fn initialize(init: GameClientInit) -> Option<Self> {
        ::std::panic::set_hook(Box::new(logging::panic_handler));

        let mut client = GameClient::default();

        if let Err(e) = client.data.assets.init(&init) {
            log_err!(e);
            return None;
        }

        Some(client)
    }

    pub fn update(&mut self, time: f64) {
        self.data.prepare_update(time);

        match self.state {
            state::GameState::Uninitialized => state::uninitialized::update(self),
            state::GameState::FinalDemo => state::final_demo::update(self, time),
        }

        output::GameOutput::update(self);
    }

    pub fn updates_ptr(&self) -> *const output::OutputIndex {
        self.output.output_index
    }

}

impl GameClient {
    pub fn on_reload(&mut self) {
        state::final_demo::init(self);
    }

    pub fn as_bytes(&mut self) -> Box<[u8]> {
        let mut writer = store::StoreWriter::new();
        self.data.store(&mut writer);
        self.state.store(&mut writer);
        writer.data()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut reader = store::StoreReader::new(bytes)?;

        let client = GameClient {
            data: data::GameData::load(&mut reader)?,
            state: state::GameState::load(&mut reader)?,
            output: output::GameOutput::default(),
        };

        Ok(client)
    }
}

/// Export the game client into an array of bytes
#[wasm_bindgen]
pub fn save(mut client: GameClient) -> Box<[u8]> {
    client.as_bytes().to_vec().into_boxed_slice()
}

/// Load the game client from an array of bytes
#[wasm_bindgen]
pub fn load(bytes: Box<[u8]>) -> GameClient {
    ::std::panic::set_hook(Box::new(logging::panic_handler));

    let client = match GameClient::from_bytes(&bytes) {
        Ok(mut client) => {
            dbg!("Game client reloaded!");
            client.on_reload();
            client
        },
        Err(e) => {
            log_err!(e);
            GameClient::default()
        }
    };

    client
}

#[wasm_bindgen]
pub fn protocol() -> String {
    output::protocol::compile()
}
