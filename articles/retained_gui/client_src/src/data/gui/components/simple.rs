//! Implementation for simple components

use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::data::assets::TextMetrics;
use crate::data::gui::components::GuiComponentView;
use crate::data::gui::generate_sprites::{generate_text, generate_borders, generate_solid_color_block};
use crate::data::gui::{GuiAssets, GuiStateStore, GuiOutputSprite};
use crate::shared::ColorRGBA8;

//
// Data
//

#[derive(Default, Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiComponentSpacer {
    pub width: f32,
    pub height: f32,
}

#[derive(Default, Copy, Clone, Immutable, IntoBytes, FromBytes)]
#[repr(align(4))]
pub struct GuiComponentSolidColorBlock {
    pub color: ColorRGBA8
}

impl GuiComponentSolidColorBlock {
    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&self, view: &GuiComponentView, callback: &mut F) {
        generate_solid_color_block(self.color, view, callback);
    }
}

#[derive(Default, Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct GuiComponentBorders {
    pub border_width: f32,
    pub border_color: ColorRGBA8,
}

impl GuiComponentBorders {
    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&self, view: &GuiComponentView, callback: &mut F) {
        generate_borders(view, self.border_width, self.border_color, callback);
    }
}

pub struct GuiComponentLabel {
    pub text: TextMetrics,
    pub scale: f32,
    pub color: ColorRGBA8,
}

impl GuiComponentLabel {
    pub fn sync_state_data(&mut self, assets: &GuiAssets, data: &GuiStateStore) {
        match data {
            GuiStateStore::String(value) => {
                self.text = assets.default_font.compute_text_metrics(value.as_str(), self.scale);
            },
            _ => {
                warn!("Unknown state data sent to label: {:?}", data.type_name())
            }
        }
    }

    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&self, view: &GuiComponentView, callback: &mut F) {
        generate_text(&self.text, view, self.color, callback);
    }
}

//
// Builder code
// 

use crate::data::gui::components::GuiComponentData;
use crate::data::gui::layout::{GuiLayoutAlignItems, FlexboxItemsLayout, FlexDirection, FlexAlignItems, FlexJustifyContent};
use crate::data::gui::{GuiBuilder, GuiState};
use crate::shared::pos;

impl<'a> GuiBuilder<'a> {
    pub fn group<F: FnOnce(&mut GuiBuilder)>(&mut self, callback: F) {
        fn check_group_layout(builder: &mut GuiBuilder) {
            let mut layout = builder.get_layout();
            if matches!(layout.align_items, GuiLayoutAlignItems::NoChildren) {
                layout.align_items = GuiLayoutAlignItems::Flexbox(FlexboxItemsLayout { 
                    children_offset: pos(0.0, 0.0),
                    direction: FlexDirection::Column,
                    align_items: FlexAlignItems::Start,
                    justify_content: FlexJustifyContent::Start,
                });
            }

            builder.set_layout(layout);
        }

        // Change the default layout for a group with an undefined layout
        check_group_layout(self);

        let component_index = self.push_parent(GuiComponentData::Group);
        callback(self);
        self.pop_parent(component_index);
    }

    pub fn toggle<F: FnOnce(&mut GuiBuilder)>(&mut self, state: GuiState<bool>, callback: F) {
        let visible = match self.inner.state_alloc.get(state) {
            Some(value) => *value,
            None => { warn!("Gui state object invalid."); return; }
        };

        let mut layout = self.get_layout();
        layout.visible = visible;
        self.set_layout(layout);

        let component_index = self.push_parent(GuiComponentData::Group);
        callback(self);
        self.pop_parent(component_index);

        self.inner.state_alloc.insert_layout_listener(state, component_index);
    }

    pub fn spacer(&mut self, width: f32, height: f32) {
        let spacer = GuiComponentSpacer { width, height };
        self.push(GuiComponentData::Spacer(spacer));
    }

    pub fn solid_color_block(&mut self, color: ColorRGBA8) {
        let block = GuiComponentSolidColorBlock { color };
        self.push(GuiComponentData::SolidColorBlock(block));
    }

    pub fn borders(&mut self, border_width: f32, border_color: ColorRGBA8) {
        let borders = GuiComponentBorders { border_width, border_color };
        self.push(GuiComponentData::Borders(borders));
    }

    pub fn label<S: AsRef<str>>(&mut self, text: S, text_scale: f32, text_color: ColorRGBA8) {
        let text = self.inner.assets.default_font.compute_text_metrics(text.as_ref(), text_scale);
        let label = GuiComponentLabel { text, scale: text_scale, color: text_color };
        self.push(GuiComponentData::Label(label));
    }

    pub fn label_dyn(&mut self, state: GuiState<String>, text_scale: f32, text_color: ColorRGBA8) {
        let label_text = match self.inner.state_alloc.get(state) {
            Some(style) => style,
            None => { warn!("Gui state object invalid."); return; }
        };

        let text = self.inner.assets.default_font.compute_text_metrics(label_text.as_str(), text_scale);
        let label = GuiComponentLabel { text, scale: text_scale, color: text_color };
        let component_index = self.push(GuiComponentData::Label(label));

        self.inner.state_alloc.insert_component_listener(state, component_index);
    }

}
