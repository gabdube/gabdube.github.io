use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::data::gui::components::GuiComponentView;
use crate::data::gui::state::{GuiStateAlloc, GuiState, GuiStateStore, ChildrenOffsetY};
use crate::data::gui::GuiOutputSprite;
use crate::shared::{AABB, SizeF32, PositionF32, size, rgba8, aabb, pos};

#[derive(Default)]
pub struct GuiComponentScrollView {
    pub content_height: f32,
    pub visible_height: f32,
    pub content_offset_vertical: f32,
    pub state_offset_y: GuiState<ChildrenOffsetY>,
}

impl GuiComponentScrollView {

    pub fn new(state_offset_y: GuiState<ChildrenOffsetY>) -> Self {
        GuiComponentScrollView { 
            content_height: 0.0,
            visible_height: 0.0,
            content_offset_vertical: 0.0,
            state_offset_y
        }
    }

    pub fn sync_state_data(&mut self, data: &GuiStateStore) {
        match data {
            GuiStateStore::ChildrenOffsetY(ChildrenOffsetY(offset)) => { self.content_offset_vertical = *offset; },
            _ => {}
        }
    }

    fn sync_scroll_value(&mut self, state: &mut GuiStateAlloc) {
        if let Some(state) = state.get_mut(self.state_offset_y) {
            *state = ChildrenOffsetY(self.content_offset_vertical);
        }
    }

    pub fn after_render(&mut self, content_height: f32, visible_height: f32) {
        self.content_height = content_height;
        self.visible_height = visible_height;
    }

    pub fn on_scroll(&mut self, state: &mut GuiStateAlloc, scroll_value: i32) -> bool {
        let scroll_amount = scroll_value as f32 / 10.0;
        let max_value = self.content_height - self.visible_height;
        let old_value = self.content_offset_vertical;
        let new_value = f32::max(0.0, f32::min(old_value + scroll_amount, max_value));

        if old_value != new_value {
            self.content_offset_vertical = new_value;
            self.sync_scroll_value(state);
            true
        } else {
            false
        }
    }

}


#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
struct ScrollbarVerticalRenderFeedback {
    pub scrollbar_rect: AABB,
    pub scrollbar_slider_rect: AABB,
}

#[derive(Default)]
struct ScrollbarInnerData {
    render_feedback: ScrollbarVerticalRenderFeedback,
    state_content_offset: GuiState<ChildrenOffsetY>,
}

pub struct GuiComponentScrollbarVertical {
    inner: Box<ScrollbarInnerData>,
    pub slider_offset: f32,
    pub slider_size: f32,
    pub content_offset: f32,
    pub content_height: f32,
    pub visible_height: f32,
    pub mouse_anchor_y: f32,
    pub content_offset_to_slider_offset: f32,
    pub grabbed: bool,
}

impl GuiComponentScrollbarVertical {
    pub fn new(state: GuiState<ChildrenOffsetY>) -> Self {
        let inner = ScrollbarInnerData {
            render_feedback: ScrollbarVerticalRenderFeedback::default(),
            state_content_offset: state,
        };

        GuiComponentScrollbarVertical {
            inner: Box::new(inner),
            slider_offset: 0.0,
            slider_size: 0.0,
            content_offset: 0.0,
            content_height: 0.0,
            visible_height: 0.0, 
            mouse_anchor_y: 0.0,
            content_offset_to_slider_offset: 0.0,
            grabbed: false,
        }
    }

    pub fn minimum_size(&self) -> SizeF32 {
        size(20.0, f32::max(self.slider_size, 10.0))
    }

    pub fn after_render(&mut self, content_height: f32, visible_height: f32) {
        self.content_height = content_height;
        self.visible_height = visible_height;
        self.compute_slider_dimensions();
    }

    pub fn sync_state_data(&mut self, data: &GuiStateStore) {
        match data {
            GuiStateStore::ChildrenOffsetY(ChildrenOffsetY(offset)) => { 
                self.content_offset = *offset;
                self.update_slider_offset();
            },
            _ => {}
        }
    }

    fn sync_scroll_value(&mut self, state: &mut GuiStateAlloc) {
        if let Some(state) = state.get_mut(self.inner.state_content_offset) {
            *state = ChildrenOffsetY(self.content_offset);
        }
    }

    pub fn update_mouse_state(&mut self, mouse_position: PositionF32, pressed: bool) -> bool {
        let grabbed_old = self.grabbed;

        if pressed {
            let slider_rect = self.inner.render_feedback.scrollbar_slider_rect;
            self.mouse_anchor_y = mouse_position.y - slider_rect.position().y;
            self.grabbed = slider_rect.point_inside(mouse_position);
        } else {
            self.grabbed = false;
        }

        self.grabbed != grabbed_old
    }

    pub fn update_mouse_position(&mut self, state: &mut GuiStateAlloc, mouse_position: PositionF32) -> bool {
        if !self.grabbed {
            return false;
        }

        let inner = &mut self.inner;
        let scrollbar = inner.render_feedback.scrollbar_rect;
        let max_offset = scrollbar.height() - self.slider_size;
        let offset_changed = mouse_position.y - scrollbar.position().y - self.mouse_anchor_y;
        self.slider_offset = f32::min(max_offset, f32::max(0.0, offset_changed));

        let old_value = self.content_offset;
        let new_value = self.slider_offset * self.content_offset_to_slider_offset;

        if old_value != new_value {
            self.content_offset = new_value;
            self.sync_scroll_value(state);
            true
        } else {
            false
        }
    }

    pub fn on_scroll(&mut self, state: &mut GuiStateAlloc, scroll_value: i32) -> bool {
        let scroll_amount = scroll_value as f32 / 10.0;
        let max_value = self.content_height - self.visible_height;
        let old_value = self.content_offset;
        let new_value = f32::max(0.0, f32::min(old_value + scroll_amount, max_value));

        if old_value != new_value {
            self.content_offset = new_value;
            self.sync_scroll_value(state);
            self.update_slider_offset();
            true
        } else {
            false
        }
    }

    fn compute_slider_dimensions(&mut self) {
        const MIN_SLIDER_SIZE: f32 = 30.0;

        let scrollbar = self.inner.render_feedback.scrollbar_rect;
        let scrollbar_height = scrollbar.height();

        if self.content_height < self.visible_height {
            self.slider_size = scrollbar_height;
            self.slider_offset = 0.0;
            self.content_offset_to_slider_offset = 0.0;
            return;
        }

        let content_diff = self.content_height - self.visible_height;
        let scrollbar_height_minus_min_height = scrollbar_height - MIN_SLIDER_SIZE;

        if content_diff < scrollbar_height_minus_min_height {
            self.slider_size = scrollbar_height - content_diff;
            self.content_offset_to_slider_offset = 1.0;
            self.update_slider_offset();
            return;
        }

        self.slider_size = MIN_SLIDER_SIZE;
        self.content_offset_to_slider_offset = content_diff / scrollbar_height_minus_min_height;
        self.update_slider_offset();
    }

    fn update_slider_offset(&mut self) {
        let feedback = self.inner.render_feedback;
        let max_offset = feedback.scrollbar_rect.height() - self.slider_size;
        let offset = self.content_offset / self.content_offset_to_slider_offset;
        self.slider_offset = f32::min(max_offset, f32::max(offset, 0.0));
    }

    pub fn generate_sprites<F: FnMut(&GuiOutputSprite)>(&mut self, view: &GuiComponentView, callback: &mut F) {
        use crate::data::gui::generate_sprites::{generate_borders, generate_solid_color_block};

        let color_border = rgba8(0, 0, 0, 255);
        let color_background = rgba8(225, 85, 85, 255);
        let color_scroll = match self.grabbed {
            true => rgba8(120, 105, 135, 255),
            false => rgba8(100, 80, 115, 255),
        };

        let scroll_pos = pos(view.position.x, view.position.y + self.slider_offset);
        let scroll_size = size(view.size.width, self.slider_size);
        self.inner.render_feedback = ScrollbarVerticalRenderFeedback {
            scrollbar_rect: aabb(view.position, view.size),
            scrollbar_slider_rect: aabb(scroll_pos, scroll_size)
        };

        let scroll_view = GuiComponentView { position: scroll_pos, size: scroll_size, ..*view };
        
        generate_solid_color_block(
            color_background,
            view,
            callback
        );

        generate_solid_color_block(
            color_scroll,
            &scroll_view,
            callback
        );

        generate_borders(
            &scroll_view,
            1.0,
            color_border,
            callback
        );

        generate_borders(
            view,
            1.0,
            color_border,
            callback
        );
    }

}


//
// Builder code
// 

use crate::data::gui::after_render_hooks::{AfterRenderHook, UpdateScrollView};
use crate::data::gui::components::GuiComponentData;
use crate::data::gui::GuiBuilder;

impl<'a> GuiBuilder<'a> {
    
    pub fn scroll_view<F: FnOnce(&mut GuiBuilder)>(&mut self, callback: F) {
        self.set_clipping();
        let offset_y = self.children_offset_y_state();
        let scroll_view = GuiComponentScrollView::new(offset_y);
        let component_index = self.push_parent(GuiComponentData::ScrollView(scroll_view));

        callback(self);

        // Adds a scrollbar component at the end
        self.layout_scrollbar_vertical();
        let scrollbar = GuiComponentScrollbarVertical::new(offset_y);
        let scrollbar_index = self.push(GuiComponentData::ScrollbarVertical(scrollbar));

        self.pop_parent(component_index);

        // The scrollview/scrollbar needs to be aware of its content height for rendering,
        // but this value is only available after rendering, so we add an "after render" hook to re-render the 
        // components on the frame once we have the right values
        self.inner.after_render.push(AfterRenderHook::UpdateScrollView(
            UpdateScrollView {
                scroll_view_index: component_index as u32,
                scroll_bar_vertical_index: scrollbar_index as u32,
                last_content_height: 0.0,
                last_visible_height: 0.0,
            }
        ));

        let pool = &mut self.inner.state_alloc;
        pool.insert_component_listener(offset_y, component_index);
        pool.insert_component_listener(offset_y, scrollbar_index);
        pool.insert_layout_listener(offset_y, component_index);
    }

}

//
// Store / Load
//

impl crate::store::StoreLoad for GuiComponentScrollView {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.content_height);
        writer.write(&self.visible_height);
        writer.write(&self.content_offset_vertical);
        self.state_offset_y.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let content_height = reader.try_read()?;
        let visible_height = reader.try_read()?;
        let content_offset_vertical = reader.try_read()?;
        let state_offset_y = GuiState::load(reader)?;
        let view = GuiComponentScrollView {
            content_height,
            visible_height,
            content_offset_vertical,
            state_offset_y
        };
        Ok(view)
    }
}

impl crate::store::StoreLoad for GuiComponentScrollbarVertical {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.inner.render_feedback);
        self.inner.state_content_offset.store(writer);

        writer.write(&self.slider_offset);
        writer.write(&self.slider_size);
        writer.write(&self.content_offset);
        writer.write(&self.content_height);
        writer.write(&self.visible_height);
        writer.write(&self.mouse_anchor_y);
        writer.write(&self.content_offset_to_slider_offset);
        writer.write_bool(self.grabbed);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let render_feedback = reader.try_read()?;
        let state_content_offset = GuiState::load(reader)?;

        let slider_offset = reader.try_read()?;
        let slider_size = reader.try_read()?;
        let content_offset = reader.try_read()?;
        let content_height = reader.try_read()?;
        let visible_height = reader.try_read()?;
        let mouse_anchor_y = reader.try_read()?;
        let content_offset_to_slider_offset = reader.try_read()?;
        let grabbed = reader.try_read_bool()?;

        let inner = ScrollbarInnerData {
            render_feedback,
            state_content_offset
        };
    
        let bar = GuiComponentScrollbarVertical {
            inner: Box::new(inner),
            slider_offset,
            slider_size,
            content_offset,
            content_height,
            visible_height,
            mouse_anchor_y,
            content_offset_to_slider_offset,
            grabbed,
        };

        Ok(bar)
    }
}

