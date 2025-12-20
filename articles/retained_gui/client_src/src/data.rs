pub mod base;
pub mod sprites;
pub mod assets;
pub mod gui;

use fnv::FnvHashMap;
use crate::error::Error;
use crate::shared::{SizeF32, PositionF32, size, pos};
use crate::GameClientInit;

use self::assets::Assets;
use self::gui::{Gui, GuiInputs};

pub type KeyState = (base::keys::Key, base::ButtonState);

pub struct GameCommon {
    pub view_size: SizeF32,

    // Time
    time: f64,
    delta_ms: u32,

    // Flags
    pub render_flags: base::RenderFlags,

    // Mouse inputs
    pub mouse_position_old: PositionF32,
    pub mouse_position: PositionF32,
    pub mouse_position_gui: PositionF32,
    pub mouse_buttons: [base::ButtonState; 3],
    pub scroll_delta_y: i32,

    // Keys inputs
    pub chars_buffer_range: u8,
    pub chars_buffer: String,

    pub keys_update_count: u8,
    pub keys_update: [KeyState; 8],

    pub keys_state: FnvHashMap<base::keys::Key, base::ButtonState>,
}

impl GameCommon {
    pub fn primary_mouse_just_pressed(&self) -> bool { self.mouse_buttons[0].just_pressed() }
    pub fn primary_mouse_just_released(&self) -> bool { self.mouse_buttons[0].just_released() }
}

#[derive(Default)]
pub struct GameData {
    pub common: GameCommon,
    pub assets: Assets,
    pub gui: Gui,
}

impl GameData {

    pub fn init(&mut self, init: &GameClientInit) -> Result<(), Error> {
        self.assets.init(init)?;
        self.gui.init_assets(&self.assets);
        self.gui.resize(init.view_size.width, init.view_size.height);
        self.common.view_size = init.view_size;
        Ok(())
    }

    pub fn reload_assets(&mut self) {
        self.gui.init_assets(&self.assets);
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.common.view_size = size(width, height);
        self.gui.resize(width, height);
        self.common.render_flags.set_update_gui();
    }

    pub fn take_render_flags(&mut self) -> base::RenderFlags {
        ::std::mem::take(&mut self.common.render_flags)
    }

    pub fn update_gui(&mut self) {
        self.common.render_flags.set_update_gui();
    }

    pub fn update_time(&mut self, new_time: f64) {
        let com = &mut self.common;
        com.delta_ms = (new_time - com.time) as u32;
        com.time = new_time;
    }

    pub fn dispatch_inputs_to_gui(&mut self) {
        let common = &self.common;
        let inputs = GuiInputs {
            delta_ms: self.common.delta_ms,
            chars_buffer: &common.chars_buffer[0..(common.chars_buffer_range as usize)],
            keys_update: &common.keys_update[0..(common.keys_update_count as usize)],
            mouse_position: common.mouse_position,
            move_mouse: common.mouse_position != common.mouse_position_old,
            primary_mouse_pressed: common.primary_mouse_just_pressed(),
            primary_mouse_released: common.primary_mouse_just_released(),
            scroll_delta_y: common.scroll_delta_y,
        };

        if self.gui.send_inputs(&inputs) {
            self.common.render_flags.set_update_gui();
        }
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.common.mouse_position_gui = pos(x, y);
        self.common.mouse_position = pos(x, y);
    }

    pub fn update_mouse_buttons(&mut self, button: u8, pressed: bool) {
        let index = button as usize;
        if index < self.common.mouse_buttons.len() {
            self.common.mouse_buttons[index] = match pressed {
                true => base::ButtonState::JustPressed,
                false => base::ButtonState::JustReleased,
            };
        }
    }

    pub fn update_keys(&mut self, key_name: &str, pressed: bool) {
        let pressed = match pressed {
            true => base::ButtonState::JustPressed,
            false => base::ButtonState::JustReleased,
        };

        if let Some(key) = base::keys::Key::from_str(key_name) {
            self.common.keys_state.insert(key, pressed);
            self.common.keys_update[self.common.keys_update_count as usize] = (key, pressed);
            self.common.keys_update_count += 1;
        }
    }

    pub fn set_chars_buffer(&mut self, buffer: String) {
        if buffer.len() > 64 {
            warn!("Chars buffer is exceeding 64 bytes! Remaining bytes will be truncated");
        }

        if buffer.len() == 0 {
            self.common.chars_buffer_range = 0;
            return;
        }

        let buffer_size = usize::min(buffer.len(), 64);
        let range = 0..buffer_size;
        self.common.chars_buffer.replace_range(range, &buffer);
        self.common.chars_buffer_range = buffer_size as u8;
    }

    pub fn finalize_update(&mut self) {
        let c = &mut self.common;
        c.mouse_buttons[0].flip();
        c.mouse_buttons[1].flip();
        c.mouse_buttons[2].flip();
        c.mouse_position_old = c.mouse_position;
        c.chars_buffer_range = 0;
        c.scroll_delta_y = 0;

        for (k, _) in &c.keys_update[0..(c.keys_update_count as usize)]{
            if let Some(state) = c.keys_state.get_mut(k) {
                state.flip();
            }
        }
        c.keys_update_count = 0;
    }
}

impl crate::store::StoreLoad for GameData {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        self.common.store(writer);
        self.assets.store(writer);
        self.gui.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, Error> {
        let mut data = GameData::default();
        data.common = GameCommon::load(reader)?;
        data.assets = Assets::load(reader)?;
        data.gui = Gui::load(reader)?;
        Ok(data)
    }
}

impl crate::store::StoreLoad for GameCommon {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.view_size);
        writer.write(&self.mouse_position_old);
        writer.write(&self.mouse_position);
        writer.write(&self.mouse_position_gui);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, Error> {
        let mut params = GameCommon::default();
        params.view_size = reader.try_read()?;
        params.mouse_position_old = reader.try_read()?;
        params.mouse_position = reader.try_read()?;
        params.mouse_position_gui = reader.try_read()?;
        Ok(params)
    }
}

impl Default for GameCommon {
    fn default() -> Self {
        GameCommon {
            view_size: size(0.0, 0.0),
            time: 0.0,
            delta_ms: 0,
            render_flags: base::RenderFlags(0),

            mouse_position_old: pos(0.0, 0.0),
            mouse_position: pos(0.0, 0.0),
            mouse_position_gui: pos(0.0, 0.0),
            mouse_buttons: [base::ButtonState::default(); 3],
            scroll_delta_y: 0,

            chars_buffer_range: 0,
            chars_buffer: {
                let mut inner = String::with_capacity(64);
                for _ in 0..64 { inner.push(' '); }
                inner
            },

            keys_update_count: 0,
            keys_update: [(base::keys::KEY_INVALID, base::ButtonState::Released); 8],

            keys_state: FnvHashMap::default(),
        }
    }
}