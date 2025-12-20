use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::data::KeyState;
use crate::shared::{PositionF32, aabb};
use super::Gui;

const MAX_HOVERED_DEPTH: usize = 12;

#[derive(Clone, Copy)]
pub(super) struct InputType(u8);

impl InputType {
    pub const NONE: Self = Self(0);
    pub const MOUSE_MOVE: Self = Self(0x1);
    pub const MOUSE_STATE: Self = Self(0x2);
    pub const SCROLL: Self = Self(0x4);
    pub const FOCUS: Self = Self(0x8);
    pub const CHARS_INPUT: Self = Self(0x10);
    pub const KEYS_INPUT: Self = Self(0x20);

    pub fn contains(&self, value: Self) -> bool {
        self.0 & value.0 == value.0
    }
}

impl ::std::ops::BitOr for InputType {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}


#[derive(Copy, Clone, Debug, Immutable, IntoBytes, FromBytes)]
pub(super) struct GuiInputState {
    pub hovered: [u32; MAX_HOVERED_DEPTH],
    pub hovered_count: u32,
    pub pressed: u32,
    pub focus: u32,    // A focused component receives keystrokes
    pub newly_rebuilt: u32,  // 1 if ui was just rebuilt, 0 otherwise. Used to force mouse move
}

impl GuiInputState {
    pub fn hovered_top(&self) -> u32 {
        match self.hovered_count == 0 {
            true => u32::MAX,
            false => self.hovered[(self.hovered_count - 1) as usize]
        }
    }
}

impl Default for GuiInputState {
    fn default() -> Self {
        GuiInputState {
            hovered: [u32::MAX; MAX_HOVERED_DEPTH],
            hovered_count: 0,
            pressed: u32::MAX,
            focus: u32::MAX,
            newly_rebuilt: 1,
        }
    }
}

pub struct GuiInputs<'a> {
    pub chars_buffer: &'a str,
    pub keys_update: &'a [KeyState],
    pub mouse_position: PositionF32,
    pub scroll_delta_y: i32,
    pub move_mouse: bool,
    pub primary_mouse_pressed: bool,
    pub primary_mouse_released: bool,
    pub delta_ms: u32,
}

//
// Inputs component dispatch
//

fn dispatch_mouse_move(
    gui: &mut Gui,
    mouse_position: PositionF32,
    component_index: u32,
    pressed: bool,
) -> bool {
    let state = &mut gui.state_alloc;
    gui.components.get_data_mut(component_index as usize)
        .on_mouse_move(state, mouse_position, pressed)
}

fn dispatch_mouse_state(
    gui: &mut Gui,
    mouse_position: PositionF32,
    component_index: u32,
    is_pressed: bool,
    is_hovered: bool,
    is_clicked: bool,
) -> bool {
    let state = &mut gui.state_alloc;
    let events = &mut gui.output_events;
    gui.components.get_data_mut(component_index as usize)
        .on_mouse_state_changed(state, events, mouse_position, is_pressed, is_hovered, is_clicked)
}

fn dispatch_scrolling(gui: &mut Gui, component_index: u32, scroll_delta_y: i32) -> bool {
    let state = &mut gui.state_alloc;
    gui.components.get_data_mut(component_index as usize)
        .on_scrolling(state, scroll_delta_y)
}

fn dispatch_focus(gui: &mut Gui, component_index: u32, selected: bool) -> bool {
    gui.components.get_data_mut(component_index as usize)
        .on_focus(selected)
}

fn dispatch_chars_input(gui: &mut Gui, component_index: u32, chars: &str) -> bool {
    let state = &mut gui.state_alloc;
    let assets = &gui.assets;
    gui.components.get_data_mut(component_index as usize)
        .on_chars_input(assets, state, chars)
}

fn dispatch_keys_input(gui: &mut Gui, component_index: u32, keys: &[KeyState]) -> bool {
    let state = &mut gui.state_alloc;
    let assets = &gui.assets;
    gui.components.get_data_mut(component_index as usize)
        .on_keys_input(assets, state, keys)
}

//
// Inputs handlers
//

// A pressed component always receive mouse move event even though the mouse might not be hover it
fn handle_mouse_move_pressed(gui: &mut Gui, mouse_position: PositionF32) -> bool {
    dispatch_mouse_move(gui, mouse_position, gui.input.pressed, true)
}

fn handle_mouse_move_default(gui: &mut Gui, mouse_position: PositionF32) -> bool {
    let [hovered_old, hovered_new] = update_hovered_components(gui, mouse_position);

    let mut state_changed = false;

    // Tell the old hovered component it is no longer hovered
    if hovered_old != hovered_new {
        if hovered_old != u32::MAX {
            state_changed |= dispatch_mouse_state(gui, mouse_position, hovered_old, false, false, false);
        }
    }

    state_changed |= handle_mouse_move_shared(gui, mouse_position);

    state_changed
}

/// Used by handle_mouse_move_default & handle_primary_mouse_released
fn handle_mouse_move_shared(gui: &mut Gui, mouse_position: PositionF32) -> bool {
    let mut hovered_count = gui.input.hovered_count as usize;
    let mut state_changed = false;
    let mut handled = false;
    while hovered_count != 0 && !handled {
        hovered_count -= 1;

        let hovered_component_index = gui.input.hovered[hovered_count];

        if respond_to_input_type(gui, hovered_component_index, InputType::MOUSE_STATE) {
            state_changed |= dispatch_mouse_state(gui, mouse_position, hovered_component_index, false, true, false);
            handled = true;
        }

        if respond_to_input_type(gui, hovered_component_index, InputType::MOUSE_MOVE) {
            state_changed |= dispatch_mouse_move(gui, mouse_position, hovered_component_index, false);
            handled = true;
        }
    }

    state_changed
}

pub(super) fn handle_mouse_move(gui: &mut Gui, mouse_position: PositionF32) -> bool {
    if gui.input.pressed != u32::MAX {
        handle_mouse_move_pressed(gui, mouse_position)
    } else {
        handle_mouse_move_default(gui, mouse_position)
    }
}

pub(super) fn handle_scrolling(gui: &mut Gui, scroll_delta_y: i32) -> bool {
    let mut hovered_count = gui.input.hovered_count as usize;
    if hovered_count == 0 {
        return false;
    }

    let mut state_changed = false;
    while hovered_count != 0 && !state_changed {
        hovered_count -= 1;
        let hovered_component_index = gui.input.hovered[hovered_count];
        if respond_to_input_type(gui, hovered_component_index, InputType::SCROLL) {
            state_changed = dispatch_scrolling(gui, hovered_component_index, scroll_delta_y);
        }
    }

    state_changed
}

pub(super) fn handle_primary_mouse_pressed(gui: &mut Gui, mouse_position: PositionF32) -> bool {
    let mut hovered_count = gui.input.hovered_count as usize;
    if hovered_count == 0 {
        gui.input.pressed = u32::MAX;
        return handle_on_focus(gui, u32::MAX);
    }

    let mut state_changed = false;
    let mut handled = false;
    while hovered_count != 0 && !handled {
        hovered_count -= 1;

        let hovered_component_index = gui.input.hovered[hovered_count];

        if respond_to_input_type(gui, hovered_component_index, InputType::FOCUS) {
            state_changed |= handle_on_focus(gui, hovered_component_index);
            handled = true;
        }

        if respond_to_input_type(gui, hovered_component_index, InputType::MOUSE_STATE) {
            state_changed |= dispatch_mouse_state(gui, mouse_position, hovered_component_index, true, true, false);
            gui.input.pressed = hovered_component_index;
            handled = true;
        }
    }

    state_changed 
}

pub(super) fn handle_primary_mouse_released(gui: &mut Gui, mouse_position: PositionF32) -> bool {
    let pressed_index = gui.input.pressed;
    gui.input.pressed = u32::MAX;
    if pressed_index == u32::MAX {
        return false;
    }

    // Because pressed component captures the mouse state, we need to refetch the hovered list
    let [_, hovered_new] = update_hovered_components(gui, mouse_position); 

    let mut state_changed = false;
    let is_clicked = hovered_new == pressed_index;
    let is_hovered = hovered_new == pressed_index;
    let is_pressed = false;
    state_changed |= dispatch_mouse_state(gui, mouse_position, pressed_index, is_pressed, is_hovered, is_clicked);
    state_changed |= handle_mouse_move_shared(gui, mouse_position);

    state_changed
}

pub(super) fn handle_chars_input(gui: &mut Gui, chars: &str) -> bool {
    if gui.input.focus == u32::MAX {
        return false;
    }

    if !respond_to_input_type(gui, gui.input.focus, InputType::CHARS_INPUT) {
        return false;
    }
    
    dispatch_chars_input(gui, gui.input.focus, chars)
}

pub(super) fn handle_keys_input(gui: &mut Gui, keys: &[KeyState]) -> bool {
    if gui.input.focus == u32::MAX {
        return false;
    }

    if !respond_to_input_type(gui, gui.input.focus, InputType::KEYS_INPUT) {
        return false;
    }

    dispatch_keys_input(gui, gui.input.focus, keys)
}

/// A gui element that is in focus captures keyboard events
/// Focus is gained when the user click on an element
/// Focus is lost when the user click on any other element
fn handle_on_focus(gui: &mut Gui, new_focus: u32) -> bool {
    if new_focus == gui.input.focus {
        return false;
    }

    let mut state_changed = false;
    let old_focus = gui.input.focus;

    if old_focus != u32::MAX {
        state_changed |= dispatch_focus(gui, old_focus, false);
    }

    if new_focus != u32::MAX && respond_to_input_type(gui, new_focus, InputType::FOCUS) {
        state_changed |= dispatch_focus(gui, new_focus, true);
    }

    gui.input.focus = new_focus;

    state_changed
}

fn update_hovered_components(gui: &mut Gui, mouse_position: PositionF32) -> [u32; 2] {
    let input = &mut gui.input;
    let hovered_old = input.hovered_top();
    let mut hovered_new = u32::MAX;

    let component_count = gui.components.len();
    let mut component_index = 0;
    let mut hovered_count = 0;
    while component_index < component_count {
        let node = gui.components.copy_node(component_index);
        let view = gui.components.copy_view(component_index);

        if view.scissor.is_zero_sized() {
            component_index += (node.descendants_count as usize) + 1;
            continue;
        }
        else if aabb(view.position, view.size).point_inside(mouse_position) {
            hovered_new = component_index as u32;
            input.hovered[hovered_count] = hovered_new;

            hovered_count += 1;
            component_index += 1;

            if hovered_count == MAX_HOVERED_DEPTH {
                warn!("Hovered chain reached maximum depth");
                break;
            }
            
        } else {
            component_index += (node.descendants_count as usize) + 1;
        }
    }

    input.hovered_count = hovered_count as u32;

    [hovered_old, hovered_new]
}

pub(super) fn respond_to_input_type(gui: &Gui, component_index: u32, input_type: InputType) -> bool {
    gui.components.get_data(component_index as usize).respond_to_input_type(input_type)
}
