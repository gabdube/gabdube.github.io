use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::data::assets::TextMetrics;
use crate::data::gui::components::GuiComponentView;
use crate::data::gui::state::{GuiStateAlloc, GuiState, LayoutOffset};
use crate::data::gui::GuiOutputSprite;
use crate::shared::{ColorRGBA8, PositionF32, SizeF32, pos, size};

#[derive(Default, Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiWindowStyle {
    pub title_bar_height: f32,
    pub title_bar_color: ColorRGBA8,
    pub title_text_scale: f32,
    pub title_text_color: ColorRGBA8,
}

struct WindowTitleBarInner {
    title_text: TextMetrics,
    style: GuiWindowStyle,
    position_state: GuiState<LayoutOffset>,
}

/// Window title bar
pub struct GuiComponentWindowTitleBar {
    inner: Box<WindowTitleBarInner>,
    size: SizeF32,
    grab_position: PositionF32,
    position: PositionF32,
    grabbed: bool,
}


impl GuiComponentWindowTitleBar {

    pub fn new(position_state: GuiState<LayoutOffset>, style: &GuiWindowStyle, text: TextMetrics) -> Self {
        let min_width = text.size.width + 5.0;
        let min_height = f32::max(style.title_bar_height, text.size.height);

        let inner = WindowTitleBarInner {
            title_text: text,
            style: *style,
            position_state,
        };
        
        GuiComponentWindowTitleBar {
            inner: Box::new(inner),
            size: size(min_width, min_height),
            grab_position: pos(0.0, 0.0),
            position: pos(0.0, 0.0),
            grabbed: false,
        }
    }

    pub fn minimum_size(&self) -> SizeF32 {
        self.size
    }

    pub fn style(&self) -> GuiWindowStyle {
        self.inner.style
    }

    pub fn text(&self) -> &TextMetrics {
        &self.inner.title_text
    }

    pub fn update_mouse_state(&mut self, mouse_position: PositionF32, pressed: bool) -> bool {
        self.grab_position = mouse_position;
        self.grabbed = pressed;
        false
    }

    pub fn update_mouse_position(&mut self, state: &mut GuiStateAlloc, mouse_position: PositionF32) -> bool {
        if !self.grabbed {
            return false;
        }

        let delta = mouse_position - self.grab_position;
        self.grab_position = mouse_position;
        self.position.x += delta.x;
        self.position.y += delta.y;

        if let Some(state) = state.get_mut(self.inner.position_state) {
            *state = LayoutOffset(self.position);
        }

        true
    }

    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&self, view: &GuiComponentView, callback: &mut F) {
        use crate::data::gui::generate_sprites::{generate_solid_color_block, generate_text};

        let style = self.style();
        generate_solid_color_block(
            style.title_bar_color,
            view,
            callback
        );

        let text = self.text();
        let mut text_view = *view;
        text_view.position.x += 5.0;
        text_view.position.y += (text_view.size.height - text.size.height) / 2.0;
        generate_text(text, &text_view, style.title_text_color, callback);
    }

}

/// Window container
pub struct GuiComponentWindow;

impl GuiComponentWindow {
    pub fn minimum_size(&self) -> SizeF32 {
        size(0.0, 0.0)
    }
}

//
// Builder code
// 

use crate::data::gui::layout::{FlexAlignItems, FlexDirection, FlexJustifyContent, FlexboxItemsLayout, GuiLayout, GuiLayoutAlignItems};
use crate::data::gui::components::GuiComponentData;
use crate::data::gui::GuiBuilder;

impl<'a> GuiBuilder<'a> {

    pub fn window<F: FnOnce(&mut GuiBuilder), S: AsRef<str>>(
        &mut self,
        style: &GuiWindowStyle,
        text: S,
        callback: F
    ) {
        fn setup_window_layout(builder: &mut GuiBuilder) -> GuiLayoutAlignItems {
            let original_layout = builder.get_layout();
            builder.set_layout(GuiLayout {
                align_self: original_layout.align_self,
                align_items: GuiLayoutAlignItems::Flexbox(FlexboxItemsLayout { 
                    children_offset: pos(0.0, 0.0),
                    direction: FlexDirection::Column,
                    align_items: FlexAlignItems::Stretch,
                    justify_content: FlexJustifyContent::Start,
                }),
                visible: true
            });

            original_layout.align_items
        }

        fn set_layout(builder: &mut GuiBuilder, align_items: GuiLayoutAlignItems) {
            let mut layout = builder.get_layout();
            layout.align_items = align_items;
            builder.set_layout(layout);
        }

        let position_state = self.layout_offset_state();
        let items_layout = setup_window_layout(self);

        // Window container
        let window = GuiComponentWindow { };
        let component_index = self.push_parent(GuiComponentData::Window(window));

        // Title bar
        self.layout_parent();
        let window_title = self.inner.assets.default_font.compute_text_metrics(text.as_ref(), style.title_text_scale);
        let title_bar = GuiComponentWindowTitleBar::new(position_state, style, window_title);
        self.push(GuiComponentData::WindowTitleBar(title_bar));

        // Window content
        self.set_clipping();
        self.layout_parent();
        set_layout(self, items_layout);
        self.group(callback);

        self.pop_parent(component_index);

        // Binding listeners
        self.inner.state_alloc.insert_layout_listener(position_state, component_index);
    }


}


//
// Store / Load
//

impl crate::store::StoreLoad for GuiComponentWindowTitleBar {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        self.inner.title_text.store(writer);
        self.inner.position_state.store(writer);
        writer.write(&self.inner.style);
        writer.write(&self.size);
        writer.write(&self.position);
        writer.write(&self.grab_position);
        writer.write_bool(self.grabbed);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let title_text = TextMetrics::load(reader)?;
        let position_state = GuiState::load(reader)?;
        let style = reader.try_read()?;
        let position = reader.try_read()?;
        let size = reader.try_read()?;
        let grab_position = reader.try_read()?;
        let grabbed = reader.try_read_bool()?;
        Ok(GuiComponentWindowTitleBar {
            inner: Box::new(WindowTitleBarInner { title_text, style, position_state }),
            size,
            position,
            grab_position,
            grabbed,
        })
        
    }
}
