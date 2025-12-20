mod state;
mod layout;
mod inputs;
mod components;
mod animations;
mod builder;

mod layout_compute;
mod generate_sprites;
mod after_render_hooks;

#[cfg(test)]
mod tests;

pub use self::animations::{GuiAnimation, GuiAnimationKeyFrame, GuiAnimationControl};
pub use self::generate_sprites::GuiOutputSprite;
pub use self::builder::GuiBuilder;
pub use self::inputs::GuiInputs;
pub use self::layout::{FlexboxItemsLayout, FlexDirection, FlexAlignItems, FlexJustifyContent};
pub use self::state::GuiState;

pub use self::components::{
    GuiImageStyle, GuiButtonStyles, GuiButtonStyle, GuiWindowStyle,
    GuiListViewItemStyles, GuiListViewItemStyle,
};

use crate::data::assets::{Assets, Font};
use crate::shared::{Scissor, SizeF32, size};
use self::after_render_hooks::AfterRenderHook;
use self::builder::GuiBuilderData;
use self::components::GuiComponents;
use self::inputs::GuiInputState;
use self::state::{GuiStateAlloc, GuiStateStore};

/// Internal storage type for user events
pub type GuiInternalEvent = ::std::num::NonZeroU32;
type GuiOutputEvents = Vec<Option<GuiInternalEvent>>;

#[derive(Default)]
struct GuiAssets {
    default_font: Font
}

pub struct Gui {
    builder: Box<GuiBuilderData>,
    assets: Box<GuiAssets>,
    state_alloc: GuiStateAlloc,
    components: GuiComponents,
    after_render: Vec<AfterRenderHook>,
    output_events: GuiOutputEvents,
    input: GuiInputState,
    view_size: SizeF32,
    iter_output_events: u32,
    delta_ms: u32,
}

impl Gui {
    pub(super) fn init_assets(&mut self, assets: &Assets) {
        self.assets.default_font = assets.roboto.clone();
    }

    pub fn build<F: FnOnce(&mut GuiBuilder)>(&mut self, callback: F) {
        GuiBuilder::build(self, callback);
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.view_size = size(width, height);
    }

    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&mut self, cb: F) -> bool {
        if self.components.is_empty() {
            return false;
        }

        let delta_ms = self.delta_ms;
        self.delta_ms = 0;

        state::sync(self);
        layout_compute::layout_compute(self);
        generate_sprites::generate_sprites(self, cb);
        after_render_hooks::after_render(self, delta_ms)
    }

    pub fn send_inputs(&mut self, inputs: &GuiInputs) -> bool {
        self.delta_ms = inputs.delta_ms;

        let mut state_changed = inputs.delta_ms != 0;

        // Always process mouse events if the UI was just rebuilt
        if inputs.move_mouse || self.input.newly_rebuilt == 1 || inputs.scroll_delta_y != 0 {
            state_changed |= inputs::handle_mouse_move(self, inputs.mouse_position);
        }

        if inputs.primary_mouse_pressed {
            state_changed |= inputs::handle_primary_mouse_pressed(self, inputs.mouse_position);
        }

        if inputs.primary_mouse_released {
            state_changed |= inputs::handle_primary_mouse_released(self, inputs.mouse_position);
        }

        if inputs.scroll_delta_y != 0 {
            state_changed |= inputs::handle_scrolling(self, inputs.scroll_delta_y);
        }

        if inputs.chars_buffer.len() > 0 {
            state_changed |= inputs::handle_chars_input(self, inputs.chars_buffer);
        }

        if inputs.keys_update.len() > 0 {
            state_changed |= inputs::handle_keys_input(self, inputs.keys_update);
        }

        self.input.newly_rebuilt = 0;

        state_changed
    }

    pub fn read_next_event<T: TryFrom<GuiInternalEvent>>(&mut self) -> Option<Result<T, T::Error>>  {
        let i = self.iter_output_events as usize;
        if i >= self.output_events.len() {
            return None;
        }

        let event = ::std::mem::take(&mut self.output_events[i]);
        match event {
            Some(value) => {
                self.iter_output_events += 1;
                Some(T::try_from(value))
            },
            None => {
                self.iter_output_events = 0;
                self.output_events.clear();
                None
            }
        }
    }

    pub fn set_state<T>(&mut self, state: GuiState<T>, value: T) {
        self.state_alloc.set(state, value);
    }

    pub fn get_state<'a, T>(&'a self, state: GuiState<T>) -> Option<&'a T> {
        self.state_alloc.get(state)
    }

    /// Returns `true` if the state is valid and the callback was executed
    pub fn mutate_state<T, CB: FnOnce(&mut T)>(&mut self, state: GuiState<T>, callback: CB) -> bool {
        match self.state_alloc.get_mut(state) {
            Some(data) => {
                callback(data);
                true
            },
            None => false
        }
    }

    pub fn base_scissor(&self) -> Scissor {
        Scissor::new(0, 0, self.view_size.width as u16, self.view_size.height as u16)
    }

}

impl Default for Gui {
    fn default() -> Self {
        Gui {
            builder: Box::default(),
            assets: Box::default(),
            state_alloc: GuiStateAlloc::default(),
            components: GuiComponents::default(),
            after_render: Vec::with_capacity(8),
            output_events: Vec::with_capacity(8),
            input: Default::default(),
            view_size: size(0.0, 0.0),
            iter_output_events: 0,
            delta_ms: 0,
        }
    }
}

impl crate::store::StoreLoad for Gui {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        self.components.store(writer);
        self.state_alloc.store(writer);
        self.after_render.store(writer);
        writer.write(&self.input);
        writer.write(&self.view_size);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let components = GuiComponents::load(reader)?;
        let state_alloc = GuiStateAlloc::load(reader)?;
        let after_render = Vec::<AfterRenderHook>::load(reader)?;
        let input = reader.try_read()?;
        let view_size = reader.try_read()?;
        let gui = Gui {
            builder: Box::default(),
            assets: Box::default(),   // Note: Gui assets are reloaded from the game assets after the load procedure
            state_alloc,
            components,
            after_render,
            output_events: Vec::with_capacity(8),
            input,
            view_size,
            iter_output_events: 0,
            delta_ms: 0,
        };

        Ok(gui)
    }
}
