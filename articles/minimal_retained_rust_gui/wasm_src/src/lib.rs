#[macro_use]
mod logging;

#[macro_use]
mod error;

mod shared;
mod store;

use wasm_bindgen::prelude::*;
use error::Error;

#[wasm_bindgen]
pub struct GameClientInit {
    pub(crate) max_texture_size: u32,
    pub(crate) view_size: shared::SizeF32,
}

#[wasm_bindgen]
impl GameClientInit {

    pub fn new() -> Self {
        GameClientInit {
            max_texture_size: 2048,
            view_size: shared::size(0.0, 0.0),
        }
    }

    pub fn max_texture_size(&mut self, value: u32) {
        self.max_texture_size = value;
    }

    pub fn view_size(&mut self, width: f32, height: f32) {
        self.view_size.width = width;
        self.view_size.height = height;
    }

}

#[wasm_bindgen]
#[derive(Default)]
pub struct GameClient {

}

#[wasm_bindgen]
impl GameClient {

    pub fn initialize(init: GameClientInit) -> Option<Self> {
        ::std::panic::set_hook(Box::new(logging::panic_handler));

        let mut client = GameClient::default();

        Some(client)
    }
    
}


impl GameClient {
    pub fn on_reload(&mut self) {
    }

    pub fn as_bytes(&mut self) -> Box<[u8]> {
        let writer = store::StoreWriter::new();
        writer.data()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let client = GameClient {
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
