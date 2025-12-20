use unicode_segmentation::UnicodeSegmentation;
use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::data::assets::TextMetrics;
use crate::data::base::{ButtonState, keys::{Key, BACKSPACE, ARROW_LEFT, ARROW_RIGHT}};
use crate::data::gui::components::GuiComponentView;
use crate::data::gui::{GuiAssets, GuiState, GuiStateAlloc, GuiStateStore, GuiOutputSprite};
use crate::shared::{ColorRGBA8, SizeF32, PositionF32, AABB, Scissor, size, rgba8, aabb, pos};

const TEXT_INPUT_MIN_SIZE: SizeF32 = size(200.0, 60.0);

#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
struct GuiComponentTextInputValueRenderFeedback {
    // Text bounds on the screen in global coordinates
    pub text_bounds: AABB,
}

pub struct GuiComponentTextInputValue {
    pub value: String,
    pub metrics: TextMetrics,
    pub color: ColorRGBA8,
    pub scale: f32,
    pub caret_position: u32,
    pub caret_offset: f32,
    pub caret_height: f32,
    pub state: GuiState<String>,
    pub text_view_offset: f32,

    // This value is written at render time in generate_sprites 
    render_feedback: GuiComponentTextInputValueRenderFeedback,
}

pub struct GuiComponentTextInput {
    pub text: Box<GuiComponentTextInputValue>,
    pub size: SizeF32,
    pub focused: bool,
}

impl GuiComponentTextInput {
    pub fn new(
        assets: &GuiAssets,
        text: String,
        state: GuiState<String>,
        text_scale: f32,
        text_color: ColorRGBA8
    ) -> Self {
        let text_metrics = assets.default_font.compute_text_metrics_aligned(&text, text_scale);
        let caret_position = text_metrics.glyphs.len() as u32;
        let caret_offset = text_metrics.size.width;
        let caret_height = assets.default_font.line_height(text_scale);
        let inner_text = GuiComponentTextInputValue {
            value: text,
            metrics: text_metrics,
            color: text_color,
            scale: text_scale,
            caret_position,
            caret_offset,
            caret_height,
            state,
            text_view_offset: 0.0,
            render_feedback: GuiComponentTextInputValueRenderFeedback::default(),
        };

        let mut text_input = GuiComponentTextInput {
            text: Box::new(inner_text),
            size: size(0.0, 0.0),
            focused: false,
        };

        text_input.compute_text_size();

        text_input
    }

    pub fn minimum_size(&self) -> SizeF32 {
        self.size
    }

    pub fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn sync_state_data(&mut self, assets: &GuiAssets, data: &GuiStateStore) {
        match data {
            GuiStateStore::String(value) => {
                self.text.value.clear();
                self.text.value.push_str(value);
                self.update_text(assets, None);
            },
            _ => {}
        }
    }

    pub fn update_mouse_state(&mut self, mouse_position: PositionF32, pressed: bool) -> bool {
        if !pressed {
            return false;
        }

        let text = &mut self.text;
        let mut local_position = mouse_position - text.render_feedback.text_bounds.position();
        local_position.x -= text.text_view_offset;
        text.caret_position = text.metrics.point_to_caret_position(local_position);
        self.update_caret(false)
    }

    fn send_backspace(&mut self, assets: &GuiAssets, state: &mut GuiStateAlloc) {
        let text = &mut self.text;
        if text.caret_position == 0 || text.value.len() == 0 {
            return;
        }

        let caret_position = (text.caret_position - 1) as usize;
        let mut iter_grapheme = UnicodeSegmentation::grapheme_indices(text.value.as_str(), true)
            .skip(caret_position);

        if let Some((offset, _)) = iter_grapheme.next() {
            text.value.remove(offset);
        } else {
            text.value.pop();
        }

        text.caret_position -= 1;

        self.update_text(assets, Some(state));
        self.update_caret(true);
    }

    fn send_arrow(&mut self, key: Key) {
        let caret_position = self.text.caret_position as i32;
        let new_position = match key == ARROW_LEFT {
            true => i32::max(caret_position - 1, 0),
            false => i32::min(caret_position + 1, self.text.metrics.glyphs.len() as i32),
        };

        self.text.caret_position = new_position as u32;
        self.update_caret(false);
    }

    pub fn send_key(&mut self, assets: &GuiAssets, state: &mut GuiStateAlloc, key: Key, pressed: ButtonState) -> bool {
        if !pressed.just_pressed() {
            return false;
        }

        match key {
            BACKSPACE => { self.send_backspace(assets, state); true },
            ARROW_LEFT | ARROW_RIGHT => { self.send_arrow(key); true },
            _ => false
        }
    }

    pub fn send_chars(&mut self, assets: &GuiAssets, state: &mut GuiStateAlloc, chars: &str) {
        if self.caret_at_end() {
            self.text.value.push_str(chars);
        } else {
            self.text.value.insert_str(self.text.caret_position as usize, chars);
        }

        self.text.caret_position += UnicodeSegmentation::graphemes(chars, true).count() as u32;
        self.update_text(assets, Some(state));
        self.update_caret(false);
    }

    fn update_text(&mut self, assets: &GuiAssets, state: Option<&mut GuiStateAlloc>) {
        let text = &mut self.text;

        if let Some(state) = state {
            if let Some(value) = state.get_mut(text.state) {
                value.clear();
                value.push_str(&text.value);
            }
        }

        text.metrics = assets.default_font.compute_text_metrics_aligned(&text.value, text.scale);
        self.compute_text_size();
    }

    fn update_caret(&mut self, backspace: bool) -> bool {
        let text = &mut self.text;
        let glyphs = &text.metrics.glyphs;
        let caret_position = text.caret_position as usize;
        let caret_offset;

        if caret_position < glyphs.len() {
            caret_offset = glyphs[caret_position].position.left - 1.0;
        } else {
            caret_offset = text.metrics.size.width;
        }

        if caret_offset == self.text.caret_offset {
            return false;
        }

        let mut text_view_offset = self.text.text_view_offset;
        let view_width = self.text.render_feedback.text_bounds.size().width;
        let caret_offset_local = caret_offset + text_view_offset;
        if backspace && text_view_offset < 0.0 {
            // The difference between the old caret position and the new one returns 
            // the distance that needs to be added to text_view_offset so that the caret stays in place
            text_view_offset += self.text.caret_offset - caret_offset;
            text_view_offset = f32::min(0.0, text_view_offset);
        } else {
            // If the new caret position is outside of the rendering box, offset the text rendering so it becomes visible
            if caret_offset_local >= view_width  {
                text_view_offset = -(caret_offset - view_width);
            } else if caret_offset_local < 0.0 {
                text_view_offset -= caret_offset_local;
            }
        }

        self.text.caret_offset = caret_offset;
        self.text.text_view_offset = text_view_offset;

        true
    }

    fn caret_at_end(&self) -> bool {
        (self.text.caret_position as usize) == self.text.metrics.glyphs.len()
    }

    fn compute_text_size(&mut self) {
        self.size = self.text.metrics.size.max(TEXT_INPUT_MIN_SIZE);
    }

    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&mut self, view: &GuiComponentView, callback: &mut F) {
        use crate::data::gui::generate_sprites::{GuiSpriteFlags, generate_solid_color_block, generate_borders, generate_text};

        const TEXT_PADDING: f32 = 6.0;
        const CARET_WIDTH: f32 = 3.0;
        const BORDER_WIDTH: f32 = 3.0;

        let [color_bg, color_borders] = match self.focused {
            true => [rgba8(230, 150, 150, 255), rgba8(200, 28, 46, 255)],
            false => [rgba8(230, 150, 150, 255), rgba8(22, 28, 46, 255)],
        };

        generate_solid_color_block(color_bg, view, callback);
        generate_borders(view, BORDER_WIDTH, color_borders, callback);

        // Compute the text view 
        let text = &mut self.text;
        let text_height = text.metrics.size.height;
        let mut text_aabb = AABB::default();
        text_aabb.left = view.position.x + TEXT_PADDING + CARET_WIDTH;
        text_aabb.top = view.position.y + ((view.size.height - text_height) / 2.0);
        text_aabb.right = view.position.x + view.size.width - TEXT_PADDING - CARET_WIDTH;
        text_aabb.bottom = text_aabb.top + text.caret_height;
        text.render_feedback.text_bounds = text_aabb;

        let mut text_view = *view;
        text_view.scissor = Scissor::from_position_and_size(text_aabb.position(), text_aabb.size());
        text_view.position = text_aabb.position() + pos(text.text_view_offset, 0.0);
        text_view.size = text_aabb.size();
        generate_text(&text.metrics, &text_view, text.color, callback);
    
        // Caret
        if self.focused {
            let caret_offset_x = TEXT_PADDING + text.text_view_offset + text.caret_offset;
            let caret_height = text.caret_height;
            let caret_offset_y = (view.size.height - caret_height) / 2.0;
            let positions = aabb(view.position, view.size);

            let sprite = GuiOutputSprite { 
                positions: AABB { 
                    left: positions.left + caret_offset_x + CARET_WIDTH,
                    top: positions.top + BORDER_WIDTH + caret_offset_y,
                    right: positions.left + caret_offset_x + CARET_WIDTH + CARET_WIDTH,
                    bottom: positions.top + BORDER_WIDTH + caret_offset_y + caret_height,
                },
                color: text.color,
                flags: GuiSpriteFlags::SOLID_COLOR,
                scissor: view.scissor,
                ..Default::default()
            };
            callback(&sprite);
        }
    }
}

//
// Builder
//

use crate::data::gui::components::GuiComponentData;
use crate::data::gui::GuiBuilder;

impl<'a> GuiBuilder<'a> {

    pub fn text_input(&mut self, text_state: GuiState<String>, text_scale: f32, text_color: ColorRGBA8) {
        let text = match self.inner.state_alloc.get(text_state) {
            Some(text) => text,
            None => {
                warn!("Gui state object invalid.");
                return;
            }
        };

        let text_input = GuiComponentTextInput::new(&self.inner.assets, text.clone(), text_state, text_scale, text_color);
        let component_index = self.push(GuiComponentData::TextInput(text_input));
        self.inner.state_alloc.insert_component_listener(text_state, component_index);
    }

}


//
// Store/Load
//

impl crate::store::StoreLoad for GuiComponentTextInputValue {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        self.state.store(writer);
        writer.write(&self.caret_position);
        writer.write(&self.caret_offset);
        writer.write(&self.caret_height);
        writer.write(&self.color);
        writer.write(&self.scale);
        writer.write(&self.text_view_offset);
        writer.write(&self.render_feedback);
        writer.write_str(&self.value);
        self.metrics.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let state = GuiState::load(reader)?;
        let caret_position = reader.try_read()?;
        let caret_offset = reader.try_read()?;
        let caret_height = reader.try_read()?;
        let color = reader.try_read()?;
        let scale = reader.try_read()?;
        let text_view_offset = reader.try_read()?;
        let render_feedback = reader.try_read()?;
        let text_value = reader.read_str().to_string();
        let text_metrics = TextMetrics::load(reader)?;
        Ok(GuiComponentTextInputValue {
            value: text_value,
            metrics: text_metrics,
            color,
            scale,
            caret_position,
            caret_offset,
            caret_height,
            state,
            text_view_offset,
            render_feedback,
        })
    }

}
