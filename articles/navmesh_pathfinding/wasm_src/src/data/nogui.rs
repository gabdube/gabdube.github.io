#![cfg(not(feature="gui"))]
#![allow(dead_code)]

use crate::data::base::DebugFlags;
use crate::shared::PositionF32;
use crate::state::{GameStateValue, GameInputType};

#[derive(Copy, Clone)]
pub enum GuiEvent {
    GameStateValueChanged(GameStateValue),
    SetDebugFlags(DebugFlags),
    SetInputType(GameInputType),
    ResetWorld,
    ResetPawnPosition,
}

pub struct Gui {
}

impl Gui {

    pub fn init(&self, _init: &crate::GameClientInit, _assets: &crate::data::Assets) -> Result<(), crate::Error> {
        Ok(())
    }

    pub fn update_time(&self, _delta: f32) {}
    pub fn resize(&mut self, _width: u32, _height: u32) { }
    pub fn update(&self) -> bool { false }
    pub fn set_state(&mut self, _state: GameStateValue, _input: GameInputType) {}
    pub fn set_debug_flags(&mut self, _flags: DebugFlags) {}
    pub fn events(&mut self) -> Vec<GuiEvent> { Vec::new() }
    pub fn clear_events(&mut self) {}
    pub fn load_font(&mut self, _assets: &crate::data::Assets) -> Result<(), crate::Error>  { Ok(()) }
    pub fn load_style(&mut self) {}
    pub fn update_mouse_position(&mut self, _x: f32, _y: f32) {}
    pub fn update_mouse_buttons(&mut self, _position: crate::shared::PositionF32, _button: u8, _pressed: bool) {}
    pub fn update_keys(&mut self, _key_name: &str, _pressed: bool) { }
    pub fn position_inside_gui(&self, _p: PositionF32) -> bool { false }
    pub fn position_outside_gui(&self, _p: PositionF32) -> bool { true }
}

impl crate::store::StoreLoad for Gui {
    fn store(&mut self, _writer: &mut crate::store::StoreWriter) {
        
    }

    fn load(_reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        Ok(Gui::default())
    }
}

impl Default for Gui {

    fn default() -> Self {
        Gui {
        }
    }

}
