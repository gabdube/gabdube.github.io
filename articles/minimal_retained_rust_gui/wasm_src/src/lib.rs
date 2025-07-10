#[macro_use]
mod logging;

#[macro_use]
mod error;

#[macro_use]
mod shared;

mod data;
mod state;
mod output;
mod store;

use fnv::FnvHashMap;
use wasm_bindgen::prelude::*;
use error::Error;
use store::StoreLoad;

#[wasm_bindgen]
pub struct GameClientInit {
    pub(crate) assets_bundle: String,
    pub(crate) view_size: shared::SizeF32,
    pub(crate) text_assets: FnvHashMap<String, String>,
}

#[wasm_bindgen]
impl GameClientInit {

    pub fn new() -> Self {
        GameClientInit {
            assets_bundle: String::new(),
            view_size: shared::size(0.0, 0.0),
            text_assets: FnvHashMap::default()
        }
    }

    pub fn set_assets_bundle(&mut self, text: String) {
        self.assets_bundle = text;
    }

    pub fn upload_text_asset(&mut self, name: String, value: String) {
        self.text_assets.insert(name, value);
    }

    pub fn view_size(&mut self, width: f32, height: f32) {
        self.view_size.width = width;
        self.view_size.height = height;
    }
    
}

#[wasm_bindgen]
#[derive(Default)]
pub struct GameClient {
    world_data: data::GameWorldData,
    state: state::GameState,
    output: output::GameOutput,
}

#[wasm_bindgen]
impl GameClient {

    pub fn initialize(init: GameClientInit) -> Option<Self> {
        ::std::panic::set_hook(Box::new(logging::panic_handler));

        let mut client = GameClient::default();

        client.world_data.data.common.view_size = init.view_size;

        if let Err(e) = client.world_data.data.assets.init(&init) {
            log_err!(e);
            return None;
        }

        Some(client)
    }

    pub fn update(&mut self, time: f64) {
        use state::GameStateValue::*;

        self.world_data.prepare_update(time);

        match self.state.value {
            Uninitialized => {
                state::init(self);
            },
            Game => {
                state::game_state::update(self);
            },
        }

        self.world_data.run_behaviours();
        self.world_data.finalize_update();

        output::GameOutput::update(self);
    }

    pub fn updates_ptr(&self) -> *const output::OutputIndex {
        self.output.output_index
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let data = &mut self.world_data.data;
        data.common.view_size = shared::size(width as f32, height as f32);
        data.common.zoom = 1.0;
        data.common.render_flags.set_update_zoom();
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.world_data.update_mouse_position(x, y);
    }

    pub fn update_mouse_buttons(&mut self, button: u8, pressed: bool) {
        self.world_data.update_mouse_buttons(button, pressed);
    }

    pub fn update_keys(&mut self, key_name: &str, pressed: bool) {
    }
}


impl GameClient {
    pub fn on_reload(&mut self) {
        state::init(self);
    }

    pub fn as_bytes(&mut self) -> Box<[u8]> {
        let mut writer = store::StoreWriter::new();
        self.world_data.store(&mut writer);
        self.state.store(&mut writer);
        writer.data()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut reader = store::StoreReader::new(bytes)?;

        let client = GameClient {
            world_data: data::GameWorldData::load(&mut reader)?,
            state: state::GameState::load(&mut reader)?,
            output: output::GameOutput::default(),
        };

        Ok(client)
    }

}

/// Export the game client into an array of bytes
#[wasm_bindgen]
pub fn save(mut client: GameClient) -> Box<[u8]> {
    client.as_bytes()
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
