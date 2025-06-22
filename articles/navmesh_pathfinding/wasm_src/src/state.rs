pub mod shared;
pub mod generation;
pub mod navigation;
pub mod pathfinding;
pub mod final_demo;

use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::data::gui::GuiEvent;
use crate::data::navigation::Triangle;
use crate::GameClient;

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, TryFromBytes, IntoBytes, Immutable)]
#[repr(u32)]
pub enum GameStateValue {
    #[default]
    Uninitialized,
    Generation,
    Navigation,
    Pathfinding,
    FinalDemo
}

#[derive(Default, PartialEq, Eq, Copy, Clone, TryFromBytes, IntoBytes, Immutable)]
#[repr(u32)]
pub enum GameInputType {
    #[default]
    Select,
    Delete,
    PlaceCastle,
    PlaceHouse,
    PlacePawn,
}

#[derive(Default, Copy, Clone)]
pub struct GameState {
    /// Only used while removing entities
    pub hovered_entity: Option<hecs::Entity>,
    /// Only used when debugging pathfinding
    pub hovered_cell: Triangle,
    pub input_type: GameInputType,
    pub value: GameStateValue,
    pub scroll_view: bool,
}

/// Propagates the events generated in the gui module to the other parts of the application
pub fn propagate_gui_events(client: &mut GameClient) {
    let events = client.data.gui.events();
    for event in events {
        match event {
            GuiEvent::GameStateValueChanged(new_state) => {
                client.state.value = new_state;
            },
            GuiEvent::SetInputType(new_input) => {
                client.data.world.clear_selected_sprites();
                client.state.input_type = new_input;
            }
            GuiEvent::SetDebugFlags(new_flags) => {
                client.data.common.debug_flags = new_flags;
            },
            GuiEvent::ResetWorld => {
                client.data.clear_sprites();
                client.data.compute_navigation();
            },
        }
    }
}

pub fn common_inputs(game: &mut GameClient) {
    let globals = game.data.common;

    if globals.middle_mouse_just_pressed() {
        game.state.scroll_view = true;
    } else if globals.middle_mouse_released() {
        game.state.scroll_view = false;
    } 
    
    if globals.secondary_mouse_just_pressed() {
        shared::secondary_mouse_actions(game);
    }

    if game.state.scroll_view {
        if let Some(delta) = globals.mouse_delta() {
            let globals = &mut game.data.common;
            globals.view_offset -= delta;
            globals.flags.set_update_view_offset();
        }
    }
}

impl crate::store::StoreLoad for GameState {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write_entity_option(self.hovered_entity);
        writer.write(&self.hovered_cell);
        writer.write(&self.input_type);
        writer.write(&self.value);
        writer.write_bool(self.scroll_view);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut state = GameState::default();

        state.hovered_entity = reader.try_read_entity_option()?;
        state.hovered_cell = reader.try_read()?;
        state.input_type = reader.try_read()?;
        state.value = reader.try_read()?;
        state.scroll_view = reader.try_read_bool()?;

        Ok(state)
    }
}
