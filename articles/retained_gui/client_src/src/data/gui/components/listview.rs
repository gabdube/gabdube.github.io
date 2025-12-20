mod list_view_builder;
pub use list_view_builder::GuiListViewBuilder;

use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::data::assets::TextMetrics;
use crate::data::gui::components::GuiComponentView;
use crate::data::gui::{GuiState, GuiStateAlloc, GuiOutputSprite, GuiOutputEvents, GuiInternalEvent};
use crate::shared::{SizeF32, ColorRGBA8, size};

#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiListViewItemStyle {
    pub background_color: ColorRGBA8,
    pub text_color: ColorRGBA8,
    pub border_color: ColorRGBA8,
}

#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiListViewItemStyles {
    pub default: GuiListViewItemStyle,
    pub hovered: GuiListViewItemStyle,
    pub pressed: GuiListViewItemStyle,
}

struct GuiComponentListViewItemInner {
    text: TextMetrics,
    styles: GuiListViewItemStyles,
    id: usize,
    selected_state: GuiState<usize>,
    item_height: f32,
    on_click: GuiInternalEvent,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GuiComponentListViewItemState {
    Default,
    Hovered,
    Pressed,
}

pub struct GuiComponentListViewItem {
    inner: Box<GuiComponentListViewItemInner>,
    size: SizeF32,
    state: GuiComponentListViewItemState,
}

impl GuiComponentListViewItem {

    pub fn new(
        id: usize,
        text: TextMetrics,
        styles: GuiListViewItemStyles,
        item_height: f32,
        selected_state: GuiState<usize>,
        on_click: GuiInternalEvent,
    ) -> Self {
        let min_size = size(text.size.width, f32::max(text.size.height, item_height));
        let inner = GuiComponentListViewItemInner {
            text,
            styles,
            id,
            selected_state,
            item_height,
            on_click,
        };

        GuiComponentListViewItem {
            inner: Box::new(inner),
            size: min_size,
            state: GuiComponentListViewItemState::Default
        }
    }

    pub fn minimum_size(&self) -> SizeF32 {
        self.size
    }

    pub fn update_mouse_state(
        &mut self,
        state: &mut GuiStateAlloc, 
        events: &mut GuiOutputEvents,
        is_pressed: bool,
        is_hovered: bool,
        is_clicked: bool
    ) -> bool {
        let old_state = self.state;
        self.state = match (is_pressed, is_hovered) {
            (true, _) => GuiComponentListViewItemState::Pressed,
            (false, true) => GuiComponentListViewItemState::Hovered,
            (false, false) => GuiComponentListViewItemState::Default
        };

        if is_pressed {
            if let Some(selected) = state.get_mut(self.inner.selected_state) {
                *selected = self.inner.id;
            }
        }
        
        if is_clicked {
            events.push(Some(self.inner.on_click));
        }

        old_state != self.state
    }

    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&self, view: &GuiComponentView, callback: &mut F) {
        use crate::data::gui::generate_sprites::{generate_solid_color_block, generate_text};

        let style = match self.state {
            GuiComponentListViewItemState::Default => self.inner.styles.default,
            GuiComponentListViewItemState::Hovered => self.inner.styles.hovered,
            GuiComponentListViewItemState::Pressed => self.inner.styles.pressed,
        };

        generate_solid_color_block(style.background_color, view, callback);
        
        let mut bottom_border_view = *view;
        bottom_border_view.position.y += view.size.height-2.0;
        bottom_border_view.size.height = 2.0;
        generate_solid_color_block(style.border_color, &bottom_border_view, callback);

        let mut text_view = *view;
        text_view.position.x += 5.0;
        text_view.position.y += (view.size.height - self.inner.text.size.height) / 2.0;
        generate_text(&self.inner.text, &text_view, style.text_color, callback);
    }
}

//
// Builder
//

use crate::data::gui::components::GuiComponentData;
use crate::data::gui::GuiBuilder;

impl<'a> GuiBuilder<'a> {

    pub fn list_view_base<F: FnOnce(&mut GuiListViewBuilder), E: Into<GuiInternalEvent>>(
        &mut self,
        on_item_clicked: E,
        selected_state: GuiState<usize>,
        styles: &GuiListViewItemStyles,
        text_size: f32,
        item_height: f32,
        callback: F
    ) {
        let component_index = self.push_parent(GuiComponentData::ListViewBase);
        let on_click = on_item_clicked.into();
        let mut builder = GuiListViewBuilder {
            inner: self.inner,
            styles: *styles,
            text_size,
            item_height,
            on_click,
            selected_state,
        };
        callback(&mut builder);

        self.pop_parent(component_index);
    }
}

impl<'a> GuiListViewBuilder<'a> {

    pub fn list_view_item<S: AsRef<str>>(&mut self, id: usize, value: S) {
        let text = self.inner.assets.default_font.compute_text_metrics_aligned(value.as_ref(), self.text_size);
        let item = GuiComponentListViewItem::new(id, text, self.styles, self.item_height, self.selected_state, self.on_click);
        self.push_item(GuiComponentData::ListViewItem(item));
    }

}

//
// Store/Load
//

impl crate::store::StoreLoad for GuiComponentListViewItem {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        self.inner.text.store(writer);
        self.inner.selected_state.store(writer);
        writer.write(&self.inner.styles);
        writer.write(&self.inner.id);
        writer.write(&self.size);
        writer.write(&self.inner.item_height);
        writer.write(&(self.inner.on_click.get()));
        writer.write(&(self.state as u32));
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let text = TextMetrics::load(reader)?;
        let selected_state = GuiState::load(reader)?;
        let styles = reader.try_read()?;
        let id = reader.try_read()?;
        let size = reader.try_read()?;
        let on_click: u32 = reader.try_read()?;
        let item_height = reader.try_read()?;
        
        let state_raw = reader.try_read::<u32>()?;
        let state = match state_raw {
            0 => GuiComponentListViewItemState::Default,
            1 => GuiComponentListViewItemState::Hovered,
            2 => GuiComponentListViewItemState::Pressed,
            _ => { return Err(assets_err!("Unknown value for GuiComponentListViewItemState::state: {state_raw}")); }
        };

        let on_click = match GuiInternalEvent::new(on_click) {
            Some(on_click) => on_click,
            None => { return Err(assets_err!("Unknown value for GuiComponentListViewItemState::on_click: {state_raw}")); }
        };

        let item = GuiComponentListViewItem {
            inner: Box::new(GuiComponentListViewItemInner {
                text,
                styles,
                id,
                selected_state,
                item_height,
                on_click
            }),
            size,
            state,
        };
        Ok(item)
    }
}
