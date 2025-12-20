mod state_store;

pub use state_store::{GuiState, ChildrenOffsetY, ChildrenOffsetX, LayoutOffset};
pub(super) use state_store::GuiStateStore;
use state_store::{GuiStateStoreWrapper, Listeners, ListenerIndex};

use std::hint::unreachable_unchecked;
use std::marker::PhantomData;

use super::animations::{GuiAnimation, GuiAnimationControl};
use super::after_render_hooks::AfterRenderHook;
use super::components::GuiComponents;
use super::{Gui, GuiAssets};

pub(super) struct GuiStateAlloc {
    store: Vec<GuiStateStoreWrapper>,
    updated: Vec<u32>,
    generation: u16,
}

impl GuiStateAlloc {

    pub fn has_updates(&self) -> bool {
        !self.updated.is_empty()
    }

    pub fn clear_updates(&mut self) {
        self.updated.clear();
    }

    pub fn clear(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.store.clear();
        self.updated.clear();
    }

    pub fn push<T>(&mut self, store: GuiStateStore) -> GuiState<T> {
        let index = self.store.len();
        if index > ListenerIndex::MAX_INDEX {
            panic!("Too many gui states, maximum of gui state object is {}", ListenerIndex::MAX_INDEX);
        }

        let state = GuiState {
            index: index as u32,
            type_id: store.type_id(),
            generation: self.generation,
            _p: PhantomData
        };

        self.store.push(GuiStateStoreWrapper {
            store,
            listeners: Listeners::default(),
        });

        state
    }

    pub fn set<T>(&mut self, state: GuiState<T>, value: T) -> bool {
        match self.get_mut(state) {
            Some(value_old) => { 
                *value_old = value;
                true
            },
            None => false
        }
    }

    pub fn get<'a, T>(&'a self, state: GuiState<T>) -> Option<&'a T> {
        self.validate_state(&state)?;

        // Safety: validate_state ensure T is the same as stored value
        let store_wrapper = &self.store[state.index as usize];
        unsafe {
            match &store_wrapper.store {
                GuiStateStore::Bool(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::Usize(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::Image(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::String(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::ChildrenOffsetY(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::ChildrenOffsetX(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::LayoutOffset(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::AnimationControl(value) => Some(::std::mem::transmute(value)),
            }
        }
    }

    pub fn get_mut<'a, T>(&'a mut self, state: GuiState<T>) -> Option<&'a mut T> {
        self.validate_state(&state)?;

        self.updated.push(state.index);

        // Safety: validate_state ensure T is the same as stored value
        let store_wrapper = &mut self.store[state.index as usize];
        unsafe {
            match &mut store_wrapper.store {
                GuiStateStore::Bool(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::Usize(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::Image(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::String(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::ChildrenOffsetY(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::ChildrenOffsetX(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::LayoutOffset(value) => Some(::std::mem::transmute(value)),
                GuiStateStore::AnimationControl(value) => Some(::std::mem::transmute(value)),
            }
        }
    }

    pub fn insert_component_listener<T>(&mut self, state: GuiState<T>, component_index: usize) {
        self.insert_listener(state, component_index, false, false);
    }

    pub fn insert_layout_listener<T>(&mut self, state: GuiState<T>, component_index: usize) {
        self.insert_listener(state, component_index, true, false);
    }

    pub fn insert_animation_listener(&mut self, state: GuiState<GuiAnimationControl>, after_render_hook_index: usize) {
        if after_render_hook_index > ListenerIndex::MAX_INDEX {
            warn!("Too many after render hook. Gui state can support up to {}", ListenerIndex::MAX_INDEX);
            return;
        }

        self.insert_listener(state, after_render_hook_index, false, true);
    }

    fn insert_listener<T>(&mut self, state: GuiState<T>, index: usize, is_layout: bool, is_animation: bool) {
        if index > ListenerIndex::MAX_INDEX {
            warn!("Too many components. Gui state can support up to {} components", ListenerIndex::MAX_INDEX);
            return;
        }

        let store_wrapper = match self.store.get_mut(state.index as usize) {
            Some(w) => w,
            None => {
                warn!("State component index {} is out of range {}", state.index, self.store.len());
                return;
            }
        };

        let listeners = &mut store_wrapper.listeners;
        match listeners.find_empty() {
            Some(listener_index) => { 
                let listener = match (is_layout, is_animation) {
                    (false, false) => ListenerIndex::new_component(index),
                    (true, false) => ListenerIndex::new_layout(index),
                    (false, true) => ListenerIndex::new_animation(index),
                    (true, true) => panic!("Invalid combination")
                };

                listeners.set(listener_index, listener);
            },
            None => {
                warn!("Too many listeners. All {} slots are already filled", listeners.len());
            }
        }
    }

    fn validate_state<T>(&self, state: &GuiState<T>) -> Option<()> {
        let store_wrapper = match self.store.get(state.index as usize) {
            Some(w) => w,
            None => {
                warn!("State component index {} is out of range {}", state.index, self.store.len());
                return None;
            }
        };

        if self.generation != state.generation {
            warn!("Mismatching data generation");
            return None;
        }

        if store_wrapper.store.type_id() != state.type_id {
            warn!("Mismatching type id");
            return None;
        }

        Some(())
    }

}

//
// State sync
//

fn sync_animation(wrapper: &mut GuiStateStoreWrapper, after_render_hooks: &mut Vec<AfterRenderHook>) {
    let animation_control = match &mut wrapper.store {
        GuiStateStore::AnimationControl(animation) => animation,
        _ => unsafe { unreachable_unchecked(); }  // store type is checked before this function
    };

    if animation_control.has_no_updates() {
        return;
    }

    for listener in wrapper.listeners.with_values() {
        let data_index = listener.index();
        let animation_state = match after_render_hooks.get_mut(data_index) {
            Some(AfterRenderHook::UpdateAnimation(animation_state)) => animation_state,
            _ => {
                warn!("listener index {} does not map to a UpdateAnimation hook", data_index);
                return;
            }
        };
        
        if animation_control.command_play() {
            animation_state.animation.flags |= GuiAnimation::PLAYING;
        }

        if animation_control.command_pause() {
            animation_state.animation.flags &= !GuiAnimation::PLAYING;
        }

        if animation_control.command_restart() {
            animation_state.current_runtime_ms = 0;
            animation_state.animation.flags |= GuiAnimation::PLAYING;
        }

    }

    animation_control.clear_updates();
}

fn sync_component(wrapper: &GuiStateStoreWrapper, assets: &GuiAssets, components: &mut GuiComponents) {
    for listener in wrapper.listeners.with_values() {
        let data_index = listener.index();
        if data_index >= components.len() {
            warn!("listener index {} does not match any component data", data_index);
            return;
        }

        if listener.is_layout() {
            components.get_layout_mut(data_index).sync_state_data(&wrapper.store);
        } else {
            components.get_data_mut(data_index).sync_state_data(assets, &wrapper.store);
        }
    }
}

pub(super) fn sync(gui: &mut Gui) {
    if !gui.state_alloc.has_updates() {
        return;
    }

    let mut updated_count = 0;
    let updated_max = gui.state_alloc.updated.len();
    while updated_count < updated_max {
        let updated_index = gui.state_alloc.updated[updated_count] as usize;

        let wrapper = &mut gui.state_alloc.store[updated_index];
        if wrapper.store.is_animation() {
            sync_animation(wrapper, &mut gui.after_render);
        } else {
            sync_component(wrapper, &gui.assets, &mut gui.components);
        }

        updated_count += 1;
    }

    gui.state_alloc.clear_updates();
}

//
// Other impls
//

impl Default for GuiStateAlloc {
    fn default() -> Self {
        GuiStateAlloc { 
            store: Vec::with_capacity(16),
            updated: Vec::with_capacity(4),
            generation: 1,
        }
    }
}

impl crate::store::StoreLoad for GuiStateAlloc {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&(self.generation as u32));

        let state_count = self.store.len();
        writer.write(&(state_count as u32));
        for i in 0..state_count {
            self.store[i].store(writer);
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let generation = (reader.try_read::<u32>()?) as u16;

        let state_count = (reader.try_read::<u32>()?) as usize;
        let mut store = Vec::with_capacity(state_count);
        for _ in 0..state_count {
            store.push(GuiStateStoreWrapper::load(reader)?);
        }

        let state = GuiStateAlloc {
            store,
            updated: Vec::with_capacity(4),
            generation,
        };

        Ok(state)
    }
}

