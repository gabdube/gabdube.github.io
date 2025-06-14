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
pub struct GameClientInit {
    pub(crate) assets_bundle: String,
    pub(crate) text_assets: FnvHashMap<String, String>,
    pub(crate) bin_assets: FnvHashMap<String, Vec<u8>>,
    pub(crate) max_texture_size: u32,
    pub(crate) view_size: shared::SizeF32,
}

#[wasm_bindgen]
impl GameClientInit {

    pub fn new() -> Self {
        GameClientInit {
            assets_bundle: String::new(),
            text_assets: FnvHashMap::default(),
            bin_assets: FnvHashMap::default(),
            max_texture_size: 2048,
            view_size: shared::size(0.0, 0.0),
        }
    }

    pub fn set_assets_bundle(&mut self, text: String) {
        self.assets_bundle = text;
    }

    pub fn upload_text_asset(&mut self, name: String, value: String) {
        self.text_assets.insert(name, value);
    }

    pub fn upload_bin_asset(&mut self, name: String, data: Vec<u8>) {
        self.bin_assets.insert(name, data);
    }

    pub fn max_texture_size(&mut self, value: u32) {
        self.max_texture_size = u32::min(value, 4096); // We don't need more than 4096px
    }

    pub fn view_size(&mut self, width: f32, height: f32) {
        self.view_size.width = width;
        self.view_size.height = height;
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

        client.data.globals.view_size = init.view_size;

        if let Err(e) = client.data.assets.init(&init) {
            log_err!(e);
            return None;
        }

        if let Err(e) = client.data.gui.init(&init, &client.data.assets) {
            log_err!(e);
            return None;
        }

        Some(client)
    }

    pub fn update(&mut self, time: f64) {
        use state::GameStateValue::*;

        if self.hidden() {
            return;
        }

        self.data.prepare_update(time);

        match self.state.value {
            Uninitialized => state::generation::init(self),
            Generation => state::generation::update(self),
            Navigation => state::navigation::update(self),
            Obstacles => state::obstacles::update(self),
            FinalDemo => state::final_demo::update(self),
        }

        state::handle_gui_events(self);

        self.data.finalize_update();

        output::GameOutput::update(self);
    }

    pub fn updates_ptr(&self) -> *const output::OutputIndex {
        self.output.output_index
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.data.globals.view_size = shared::size(width as f32, height as f32);
        self.data.gui.resize(width, height);
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.data.update_mouse_position(x, y);
    }

    pub fn update_mouse_buttons(&mut self, button: u8, pressed: bool) {
        self.data.update_mouse_buttons(button, pressed);
    }

    pub fn update_keys(&mut self, key_name: &str, pressed: bool) {
        self.data.gui.update_keys(key_name, pressed);
    }

}

impl GameClient {
    pub fn on_reload(&mut self) {
        self.data.clear_sprites();
        state::generation::init(self);
    }

    pub fn as_bytes(&mut self) -> Box<[u8]> {
        let mut writer = store::StoreWriter::new();
        self.data.store(&mut writer);
        self.state.store(&mut writer);
        writer.data()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut reader = store::StoreReader::new(bytes)?;

        let mut client = GameClient {
            data: data::GameData::load(&mut reader)?,
            state: state::GameState::load(&mut reader)?,
            output: output::GameOutput::default(),
        };

        client.data.gui.set_state(client.state.value, client.state.input_type);
        client.data.gui.set_debug_flags(client.data.globals.debug_flags);

        Ok(client)
    }

    pub fn hidden(&mut self) -> bool {
        self.data.globals.view_size.width == 0.0
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
