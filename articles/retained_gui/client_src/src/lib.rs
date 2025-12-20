#[macro_use]
mod logging;

#[macro_use]
mod error;

#[macro_use]
mod shared;

mod store;

mod data;
mod state;
mod output;


use fnv::FnvHashMap;
use wasm_bindgen::prelude::*;
use crate::error::Error;
use crate::state::GameState;
use crate::store::StoreLoad;

#[wasm_bindgen]
pub struct GameClientInit {
    pub(crate) assets_bundle: String,
    pub(crate) view_size: shared::SizeF32,
    pub(crate) text_assets: FnvHashMap<String, String>,
    pub(crate) bin_assets: FnvHashMap<String, Vec<u8>>,
}

#[wasm_bindgen]
impl GameClientInit {
    pub fn new() -> Self {
        GameClientInit {
            assets_bundle: String::new(),
            view_size: shared::size(0.0, 0.0),
            text_assets: FnvHashMap::default(),
            bin_assets: FnvHashMap::default(),
        }
    }

    pub fn set_assets_bundle(&mut self, text: String) {
        self.assets_bundle = text;
    }

    pub fn upload_text_asset(&mut self, name: String, value: String) {
        self.text_assets.insert(name, value);
    }

    pub fn upload_bin_asset(&mut self, name: String, value: Vec<u8>) {
        self.bin_assets.insert(name, value);
    }
    
    pub fn view_size(&mut self, width: f32, height: f32) {
        self.view_size.width = width;
        self.view_size.height = height;
    }
}


#[wasm_bindgen]
#[derive(Default)]
pub struct GameClient {
    data: data::GameData,
    state: GameState,
    output: output::GameOutput,
}

#[wasm_bindgen]
impl GameClient {

    pub fn initialize(init: GameClientInit) -> Option<Self> {
        ::std::panic::set_hook(Box::new(logging::panic_handler));

        let mut client = GameClient {
            data: data::GameData::default(),
            state: GameState::default(),
            output: output::GameOutput::default(),
        };

        if let Err(e) = client.data.init(&init) {
            log_err!(e);
            return None;
        }

        dbg!("Client initialized");

        Some(client)
    }

    pub fn update(&mut self, time: f64) {
        self.data.update_time(time);
        self.data.dispatch_inputs_to_gui();

        match &mut self.state {
            GameState::Uninitialized => {
                self.state = GameState::Running(state::running::RunningState::default()); 
                state::running::rebuild_running_gui(self);
            },
            GameState::Running(_) => state::running::update(self),

        }

        self.data.finalize_update();

        output::GameOutput::update(self);
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.data.resize(width, height);
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.data.update_mouse_position(x, y);
    }

    pub fn update_mouse_buttons(&mut self, button: u8, pressed: bool) {
        self.data.update_mouse_buttons(button, pressed);
    }

    pub fn update_keys(&mut self, key_name: &str, pressed: bool) {
        self.data.update_keys(key_name, pressed);
    }

    pub fn push_chars_buffer(&mut self, buffer: String) {
        self.data.set_chars_buffer(buffer);
    }

    pub fn update_scroll_value(&mut self, scroll_y: i32) {
        self.data.common.scroll_delta_y = scroll_y;
    }

    pub fn updates_ptr(&self) -> *const output::OutputIndex {
        self.output.output_index
    }

}

impl GameClient {

    pub fn on_reload(&mut self) {
        self.data.reload_assets();
        state::running::rebuild_running_gui(self);
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
    client.as_bytes()
}

/// Load the game client from an array of bytes
#[wasm_bindgen]
pub fn load(bytes: Box<[u8]>) -> GameClient {
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
