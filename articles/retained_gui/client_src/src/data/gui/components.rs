mod simple;
mod image;
mod button;
mod text_input;
mod listview;
mod scrollview;
mod window;

pub use self::simple::{GuiComponentSpacer, GuiComponentSolidColorBlock, GuiComponentBorders, GuiComponentLabel};
pub use self::image::GuiImageStyle;
pub use self::button::{GuiButtonStyle, GuiButtonStyles};
pub use self::text_input::{GuiComponentTextInput, GuiComponentTextInputValue};
pub use self::listview::{GuiListViewItemStyles, GuiListViewItemStyle};
pub use self::window::{GuiWindowStyle};

pub(super) use self::image::GuiComponentImage;
pub(super) use self::button::{GuiComponentButton, GuiComponentButtonState};
pub(super) use self::listview::{GuiComponentListViewItem };
pub(super) use self::scrollview::{GuiComponentScrollView, GuiComponentScrollbarVertical};
pub(super) use self::window::{GuiComponentWindow, GuiComponentWindowTitleBar};

use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use std::alloc::{Layout, alloc, dealloc};

use crate::data::assets::TextMetrics;
use crate::data::KeyState;
use crate::shared::{PositionF32, SizeF32, Scissor};

use super::inputs::InputType;
use super::layout::GuiLayout;
use super::state::GuiStateStore;
use super::{GuiAssets, GuiStateAlloc, GuiInternalEvent, GuiOutputEvents};


#[derive(Default, Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiNode {
    pub children_count: u32,
    pub descendants_count: u32,
    pub clip: u32,
}

impl GuiNode {
    pub fn clip() -> Self {
        GuiNode { 
            children_count: 0,
            descendants_count: 0,
            clip: 1
        }
    }
}

#[derive(Copy, Clone, Default, Immutable, IntoBytes, FromBytes)]
pub struct GuiComponentView {
    /// Position of the component in the gui
    pub position: PositionF32,
    /// Size of the component
    pub size: SizeF32,
    /// Size of the component children
    pub items_size: SizeF32,
    /// Scissor of the view
    pub scissor: Scissor,
}

impl GuiComponentView {
    pub fn clipped(&self) -> bool {
        let left = self.position.x ;
        let right = self.position.x + self.size.width;
        let scissor_left = self.scissor.x as f32;
        let scissor_right = (self.scissor.x + self.scissor.width) as f32;
        if left > scissor_right || right < scissor_left {
            return true
        }

        let top = self.position.y;
        let bottom = self.position.y + self.size.height;
        let scissor_top = self.scissor.y as f32;
        let scissor_bottom = (self.scissor.y + self.scissor.height) as f32;
        if top > scissor_bottom || bottom < scissor_top {
            return true
        }

        false
    }
}

pub enum GuiComponentData {
    Group,
    Spacer(GuiComponentSpacer),
    SolidColorBlock(GuiComponentSolidColorBlock),
    Borders(GuiComponentBorders),
    Image(GuiComponentImage),
    Label(GuiComponentLabel),
    Button(GuiComponentButton),
    TextInput(GuiComponentTextInput),
    ListViewBase,
    ListViewItem(GuiComponentListViewItem),
    ScrollView(GuiComponentScrollView),
    ScrollbarVertical(GuiComponentScrollbarVertical),
    Window(GuiComponentWindow),
    WindowTitleBar(GuiComponentWindowTitleBar),
}

impl GuiComponentData {

    pub(super) fn sync_state_data(&mut self, assets: &GuiAssets, data: &GuiStateStore) {
        match self {
            Self::Image(image) => { image.sync_state_data(data); },
            Self::Label(label) => { label.sync_state_data(assets, data); }
            Self::TextInput(text_input) => { text_input.sync_state_data(assets, data); }
            Self::ScrollView(view) => { view.sync_state_data(data); }
            Self::ScrollbarVertical(bar) => { bar.sync_state_data(data); }
            _ => {}
        }
    }

    pub(super) fn on_mouse_move(
        &mut self,
        state: &mut GuiStateAlloc,
        mouse_position: PositionF32,
        pressed: bool
    ) -> bool {
        match self {
            GuiComponentData::TextInput(text_input) => {
                text_input.update_mouse_state(mouse_position, pressed)
            },
            GuiComponentData::ScrollbarVertical(bar) => {
                bar.update_mouse_position(state, mouse_position)
            },
            GuiComponentData::WindowTitleBar(bar) => {
                bar.update_mouse_position(state, mouse_position)
            },
            _ => false
        }
    }

    pub(super) fn on_mouse_state_changed(
        &mut self,
        state: &mut GuiStateAlloc,
        events: &mut GuiOutputEvents,
        mouse_position: PositionF32,
        is_pressed: bool,
        is_hovered: bool,
        is_clicked: bool,
    ) -> bool {
        match self {
            GuiComponentData::Button(button) => {
                button.update_mouse_state(events, is_pressed, is_hovered, is_clicked)
            },
            GuiComponentData::TextInput(text_input) => {
                text_input.update_mouse_state(mouse_position, is_pressed)
            },
            GuiComponentData::ScrollbarVertical(bar) => {
                bar.update_mouse_state(mouse_position, is_pressed)
            },
            GuiComponentData::ListViewItem(item) => {
                item.update_mouse_state(state, events, is_pressed, is_hovered, is_clicked)
            },
            GuiComponentData::WindowTitleBar(bar) => {
                bar.update_mouse_state(mouse_position, is_pressed)
            }
            _ => false
        }
    }

    pub(super) fn on_scrolling(&mut self, state: &mut GuiStateAlloc, scroll_delta_y: i32) -> bool {
        match self {
            GuiComponentData::ScrollView(view) => view.on_scroll(state, scroll_delta_y),
            GuiComponentData::ScrollbarVertical(bar) => bar.on_scroll(state, scroll_delta_y),
            _ => false
        }
    }

    pub(super) fn on_focus(&mut self, selected: bool) -> bool {
        match self {
            GuiComponentData::TextInput(text_input) => {
                text_input.on_focus(selected);
                true
            },
            _ => false
        }
    }

    pub(super) fn on_chars_input(&mut self, assets: &GuiAssets, state: &mut GuiStateAlloc, chars: &str) -> bool {
        match self {
            GuiComponentData::TextInput(text_input) => {
                text_input.send_chars(assets, state, chars);
                true
            },
            _ => false
        }
    }

    pub(super) fn on_keys_input(&mut self, assets: &GuiAssets, state: &mut GuiStateAlloc, keys: &[KeyState]) -> bool {
        match self {
            GuiComponentData::TextInput(text_input) => {
                let mut updated = false;
                for &(key, pressed) in keys {
                    updated |= text_input.send_key(assets, state, key, pressed);
                }
                updated
            },
            _ => false
        }
    }

    pub(super) fn respond_to_input_type(&self, input_type: InputType) -> bool {
        let flags = match self {
            GuiComponentData::Group |
            GuiComponentData::Spacer(_) |
            GuiComponentData::SolidColorBlock(_) |
            GuiComponentData::Borders(_) |
            GuiComponentData::Image(_) |
            GuiComponentData::Label(_) |
            GuiComponentData::ListViewBase => InputType::NONE,
            GuiComponentData::Button(_) => InputType::MOUSE_MOVE | InputType::MOUSE_STATE,
            GuiComponentData::TextInput(_) => InputType::MOUSE_MOVE | InputType::MOUSE_STATE | InputType::FOCUS | InputType::CHARS_INPUT | InputType::KEYS_INPUT,
            GuiComponentData::ListViewItem(_) => InputType::MOUSE_MOVE | InputType::MOUSE_STATE,
            GuiComponentData::ScrollView(_) => InputType::SCROLL | InputType::MOUSE_MOVE | InputType::MOUSE_STATE,
            GuiComponentData::ScrollbarVertical(_) => InputType::MOUSE_MOVE | InputType::MOUSE_STATE | InputType::SCROLL,
            GuiComponentData::Window(_) => InputType::MOUSE_MOVE | InputType::MOUSE_STATE,
            GuiComponentData::WindowTitleBar(_) => InputType::MOUSE_MOVE | InputType::MOUSE_STATE ,
        };

        flags.contains(input_type)
    }

}

pub struct GuiComponents {
    base: *mut u8,
    len: u32,
    capacity: u32,
    base_capacity: u32,
    views_offset_bytes: u32,
    layout_offset_bytes: u32,
    data_offset_bytes: u32,
}

impl GuiComponents {
    pub(super) fn with_capacity(cap: usize) -> Self {
        let nodes_offset_bytes = 0;
        let views_offset_bytes = nodes_offset_bytes + (size_of::<GuiNode>() * cap) as u32;
        let layout_offset_bytes = views_offset_bytes + (size_of::<GuiComponentView>() * cap) as u32;
        let data_offset_bytes = layout_offset_bytes + (size_of::<GuiLayout>() * cap) as u32;
        let total_capacity = crate::shared::align_up(data_offset_bytes as usize + (size_of::<GuiComponentData>() * cap), size_of::<usize>());

        let base_capacity = total_capacity / size_of::<usize>();
        let base = unsafe { alloc(Layout::array::<usize>(base_capacity).unwrap()) };

        GuiComponents {
            base,
            len: 0,
            capacity: cap as u32,
            base_capacity: base_capacity as u32,
            views_offset_bytes,
            layout_offset_bytes,
            data_offset_bytes,
        }
    }

    fn realloc(&mut self) {
        use std::ptr;

        let new_capacity = (self.capacity + 32) as usize;
        let mut old = Self::with_capacity(new_capacity);
        ::std::mem::swap(self, &mut old);

        self.len = old.len;

        unsafe {
            let len = self.len as usize;
            ptr::copy_nonoverlapping(old.nodes_ptr(), self.nodes_ptr(), len);
            ptr::copy_nonoverlapping(old.views_ptr(), self.views_ptr(), len);
            ptr::copy_nonoverlapping(old.layouts_ptr(), self.layouts_ptr(), len);
            ptr::copy_nonoverlapping(old.data_ptr(), self.data_ptr(), len);
        }
       
        old.len = 0;  // Setting the len to 0 so we don't drop components data when dropping the value (see fn clear)
        drop(old);
    }

    fn free(&mut self) {
        self.clear();

        unsafe {
            dealloc(self.base as _, Layout::array::<usize>(self.base_capacity as usize).unwrap());
        }
    }

    pub(super) fn clear(&mut self) {
        if self.len > 0 {
            for i in 0..(self.len as usize) {
                unsafe { drop(self.data_ptr().add(i).read()); }
            }
        }

        self.len = 0;
    }

    pub(super) fn is_empty(&self) -> bool { self.len == 0 }
    pub(super) fn len(&self) -> usize { self.len as usize }

    pub(super) fn push(
        &mut self,
        node: GuiNode,
        layout: GuiLayout,
        data: GuiComponentData
    ) -> usize {
        if self.len == self.capacity {
            self.realloc();
        }

        let offset = self.len as usize;
        self.len += 1;

        unsafe {
            self.nodes_ptr().add(offset).write(node);
            self.layouts_ptr().add(offset).write(layout);
            self.data_ptr().add(offset).write(data);
            self.views_ptr().add(offset).write(GuiComponentView::default());
        }

        offset
    }

    fn nodes_ptr(&self) -> *mut GuiNode { self.base as *mut GuiNode }
    fn layouts_ptr(&self) -> *mut GuiLayout { (unsafe { self.base.add(self.layout_offset_bytes as usize) }) as *mut GuiLayout }
    fn data_ptr(&self) -> *mut GuiComponentData { (unsafe { self.base.add(self.data_offset_bytes as usize) }) as *mut GuiComponentData }
    fn views_ptr(&self) -> *mut GuiComponentView { (unsafe { self.base.add(self.views_offset_bytes as usize) }) as *mut GuiComponentView }

    pub(super) fn copy_node(&self, index: usize) -> GuiNode {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { self.nodes_ptr().add(index).read() }
    }

    #[cfg(test)]
    pub(super) fn get_node(&self, index: usize) -> &GuiNode {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { &*self.nodes_ptr().add(index) }
    }

    pub(super) fn get_node_mut(&mut self, index: usize) -> &mut GuiNode {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { &mut *self.nodes_ptr().add(index) }
    }

    pub(super) fn copy_view(&self, index: usize) -> GuiComponentView {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { self.views_ptr().add(index).read() }
    }

    pub(super) fn get_view(&self, index: usize) -> &GuiComponentView {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { &*self.views_ptr().add(index) }
    }

    pub(super) fn set_view(&self, index: usize, view: GuiComponentView) {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { self.views_ptr().add(index).write(view); }
    }

    pub(super) fn copy_layout(&self, index: usize) -> GuiLayout {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { self.layouts_ptr().add(index).read() }
    }

    pub(super) fn get_layout_mut(&mut self, index: usize) -> &mut GuiLayout {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { &mut *self.layouts_ptr().add(index) }
    }

    pub(super) fn get_data(&self, index: usize) -> &GuiComponentData {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { &*self.data_ptr().add(index) }
    }

    pub(super) fn get_data_mut(&self, index: usize) -> &mut GuiComponentData {
        assert!(index < (self.len as usize), "index is out of scope");
        unsafe { &mut *self.data_ptr().add(index) }
    }
}

impl Drop for GuiComponents {
    fn drop(&mut self) {
       self.free();
    }
}

impl Default for GuiComponents {
    fn default() -> Self {
        GuiComponents::with_capacity(32)
    }
}

impl crate::store::StoreLoad for GuiComponents {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        let component_count = self.len as usize;

        writer.write(&self.len);

        for i in 0..component_count {
            let node = self.copy_node(i);
            writer.write(&node);
        }

        for i in 0..component_count {
            let view = self.copy_view(i);
            writer.write(&view);
        }

        for i in 0..component_count {
            self.copy_layout(i).store(writer);
        }

        for i in 0..component_count {
            self.get_data(i).store(writer);
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let component_count = reader.try_read::<u32>()? as usize;
        let component_capacity = usize::max(32, component_count);
        let mut components = GuiComponents::with_capacity(component_capacity);
        components.len = component_count as u32;

        for i in 0..component_count {
            let node = reader.try_read()?;
            unsafe { ::std::ptr::write(components.nodes_ptr().add(i), node); }
        }

        for i in 0..component_count {
            let view = reader.try_read()?;
            unsafe { ::std::ptr::write(components.views_ptr().add(i), view); }
        }

        for i in 0..component_count {
            let layout = GuiLayout::load(reader)?;
            unsafe { ::std::ptr::write(components.layouts_ptr().add(i), layout); }
        }

        for i in 0..component_count {
            let data = GuiComponentData::load(reader)?;
            unsafe { ::std::ptr::write(components.data_ptr().add(i), data); }
        }

        Ok(components)
    }
}

impl crate::store::StoreLoad for GuiComponentData {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        match self {
            Self::Group => { writer.write(&1u32); },
            Self::Spacer(spacer) => { 
                writer.write(&2u32);
                writer.write(spacer);
            }
            Self::SolidColorBlock(block) => {
                writer.write(&3u32);
                writer.write(block);
            }
            Self::Borders(borders) => {
                writer.write(&4u32);
                writer.write(borders);
            }
            Self::Image(image) => {
                writer.write(&5u32);
                writer.write(image);
            }
            Self::Label(label) => {
                writer.write(&6u32);
                label.text.store(writer);
                writer.write(&label.scale);
                writer.write(&label.color);
            }
            Self::Button(button) => {
                writer.write(&7u32);
                writer.write(&button.size);
                writer.write(&(button.state as u32));
                writer.write(&(button.on_click.get()));
                writer.write(button.styles.as_ref());
                button.text.store(writer);
            }
            Self::TextInput(text_input) => {
                writer.write(&8u32);
                writer.write(&text_input.size);
                writer.write_bool(text_input.focused);
                text_input.text.store(writer);
            }
            Self::ListViewBase => {
                writer.write(&9u32);
            }
            Self::ListViewItem(item) => {
                writer.write(&10u32);
                item.store(writer);
            }
            Self::ScrollView(view) => {
                writer.write(&11u32);
                view.store(writer);
            },
            Self::ScrollbarVertical(bar) => {
                writer.write(&12u32);
                bar.store(writer);
            },
            Self::Window(_) => {
                writer.write(&13u32);
            },
            Self::WindowTitleBar(bar) => {
                writer.write(&14u32);
                bar.store(writer);
            }
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let id: u32 = reader.try_read()?;
        match id {
            1 => Ok(GuiComponentData::Group),
            2 => Ok(GuiComponentData::Spacer(reader.try_read()?)),
            3 => Ok(GuiComponentData::SolidColorBlock(reader.try_read()?)),
            4 => Ok(GuiComponentData::Borders(reader.try_read()?)),
            5 => Ok(GuiComponentData::Image(reader.try_read()?)),
            6 => {
                let label = GuiComponentLabel {
                    text: TextMetrics::load(reader)?,
                    scale: reader.try_read()?,
                    color: reader.try_read()?,
                };
                Ok(GuiComponentData::Label(label))
            }
            7 => {
                let size = reader.try_read()?;
                let state = GuiComponentButtonState::from_u32(reader.try_read::<u32>()?)?;
                let on_click_raw: u32 = reader.try_read()?;
                let styles = reader.try_read()?;
                let text = TextMetrics::load(reader)?;

                let on_click = match GuiInternalEvent::new(on_click_raw) {
                    Some(on_click) => on_click,
                    None => { return Err(assets_err!("Unknown value for GuiComponentData::on_click: {on_click_raw}")); }
                };

                let button = GuiComponentButton {
                    styles: Box::new(styles),
                    text: Box::new(text),
                    size,
                    on_click,
                    state,
                };
                Ok(GuiComponentData::Button(button))
            }
            8 => {
                let size = reader.try_read()?;
                let focused = reader.try_read_bool()?;
                let text = crate::data::gui::components::GuiComponentTextInputValue::load(reader)?;
                let text_input = GuiComponentTextInput {
                    text: Box::new(text),
                    size,
                    focused,
                };
                Ok(GuiComponentData::TextInput(text_input))
            }
            9 => {
                Ok(GuiComponentData::ListViewBase)
            }
            10 => {
                let item = GuiComponentListViewItem::load(reader)?;
                Ok(GuiComponentData::ListViewItem(item))
            }
            11 => {
                let view = GuiComponentScrollView::load(reader)?;
                Ok(GuiComponentData::ScrollView(view))
            }
            12 => {
                let scrollbar = GuiComponentScrollbarVertical::load(reader)?;
                Ok(GuiComponentData::ScrollbarVertical(scrollbar))
            },
            13 => {
                let window = GuiComponentWindow { };
                Ok(GuiComponentData::Window(window))
            },
            14 => {
                let bar = GuiComponentWindowTitleBar::load(reader)?;
                Ok(GuiComponentData::WindowTitleBar(bar))
            },
            _ => {
                Err(save_err!("Unknown component identifier {}. Data is most likely corrupted", id))
            }
        }
    }
}
