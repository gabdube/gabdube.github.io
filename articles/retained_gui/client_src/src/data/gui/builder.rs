use crate::data::assets::Texture;
use crate::data::sprites::StaticSprite;
use crate::shared::{SizeF32, pos};

use super::after_render_hooks::AfterRenderHook;
use super::animations::{GuiAnimation, GuiAnimationPlayState, GuiAnimationControl};
use super::components::*;
use super::layout::*;
use super::state::{GuiState, GuiStateStore, ChildrenOffsetY, LayoutOffset};
use super::Gui;


pub(super) struct GuiBuilderStack {
    pub children_count: u32,
    pub descendants_count: u32,
}

pub(super) struct GuiBuilderData {
    stack: Vec<GuiBuilderStack>,
    layout: GuiLayout,
    node: GuiNode,
    next_animation: Option<GuiAnimation>,
    next_animation_control: Option<GuiState<GuiAnimationControl>>,
}

impl GuiBuilderData {
    pub(super) fn update_parent_children_count(&mut self, descendants_count: u32) {
        if let Some(build_stack) = self.stack.last_mut() {
            build_stack.children_count += 1;
            build_stack.descendants_count += 1 + descendants_count;
        }
    }
}

pub struct GuiBuilder<'a> {
    pub(super) inner: &'a mut Gui
}

impl<'a> GuiBuilder<'a> {

    fn reset_gui(&mut self) {
        let inner = &mut self.inner;
        inner.builder.layout = GuiLayout::default();
        inner.input = super::GuiInputState::default();
        inner.state_alloc.clear();
        inner.components.clear();
        inner.after_render.clear();
    }

    fn initialize_animations(&mut self) {
        let components = &mut self.inner.components;
        let after_render = &mut self.inner.after_render;
        for hook in after_render.iter_mut() {
            if let AfterRenderHook::UpdateAnimation(animation_state) = hook {
                animation_state.apply(0, components);
            }
        }
    }

    pub(super) fn build<F: FnOnce(&mut GuiBuilder)>(gui: &'a mut Gui, callback: F) {
        let mut builder = GuiBuilder { inner: gui };
        builder.reset_gui();
        callback(&mut builder);
        builder.initialize_animations();
    }

    //
    // State
    //

    pub fn image_state(&mut self, texture: Texture, sprite: StaticSprite) -> GuiState<GuiImageStyle> {
        let store = GuiStateStore::Image(GuiImageStyle { texture, sprite });
        self.inner.state_alloc.push(store)
    }

    pub fn string_state<S: Into<String>>(&mut self, value: S) -> GuiState<String> {
        self.inner.state_alloc.push(GuiStateStore::String(value.into()))
    }

    pub fn bool_state(&mut self, value: bool) -> GuiState<bool> {
        self.inner.state_alloc.push(GuiStateStore::Bool(value))
    }

    pub fn usize_state(&mut self, value: usize) -> GuiState<usize> {
        self.inner.state_alloc.push(GuiStateStore::Usize(value))
    }

    pub fn animation_state(&mut self, value: GuiAnimationControl) -> GuiState<GuiAnimationControl> {
        self.inner.state_alloc.push(GuiStateStore::AnimationControl(value))
    }

    pub(super) fn children_offset_y_state(&mut self) -> GuiState<ChildrenOffsetY> {
        self.inner.state_alloc.push(GuiStateStore::ChildrenOffsetY(ChildrenOffsetY(0.0)))
    }

    pub(super) fn layout_offset_state(&mut self) -> GuiState<LayoutOffset> {
        self.inner.state_alloc.push(GuiStateStore::LayoutOffset(LayoutOffset(pos(0.0, 0.0))))
    }

    //
    // Layout
    //

    pub fn layout_background(&mut self) {
        self.inner.builder.layout.align_self = GuiLayoutAlignSelf::background();
    }

    pub fn layout_center(&mut self) {
        self.inner.builder.layout.align_self = GuiLayoutAlignSelf::center();
    }

    pub fn layout_center_min_size(&mut self, size: SizeF32) {
        self.inner.builder.layout.align_self = GuiLayoutAlignSelf::center_min_size(size);
    }

    pub fn layout_parent(&mut self) {
        self.inner.builder.layout.align_self = GuiLayoutAlignSelf::parent();
    }

    pub fn layout_parent_fixed_width(&mut self, min_width: f32) {
        self.inner.builder.layout.align_self = GuiLayoutAlignSelf::parent_fixed_width(min_width);
    }

    pub fn layout_parent_fixed_size(&mut self, size: SizeF32) {
        self.inner.builder.layout.align_self = GuiLayoutAlignSelf::parent_fixed_size(size);
    }

    pub fn layout_scrollbar_vertical(&mut self) {
        self.inner.builder.layout.align_self = GuiLayoutAlignSelf::scrollbar_vertical();
    }

    pub fn layout_items_flex(&mut self, flex: FlexboxItemsLayout) {
        self.inner.builder.layout.align_items = GuiLayoutAlignItems::Flexbox(flex);
    }

    //
    // Animations
    //
    
    #[allow(dead_code)]
    pub fn animate(&mut self, animation: GuiAnimation) {
        self.inner.builder.next_animation = Some(animation);
    }

    pub fn animate_dyn(&mut self, control: GuiState<GuiAnimationControl>, animation: GuiAnimation) {
        self.inner.builder.next_animation = Some(animation);
        self.inner.builder.next_animation_control = Some(control);
    }
    
    //
    // Helpers
    //

    pub(super) fn set_layout(&mut self, layout: GuiLayout) {
        self.inner.builder.layout = layout;
    }

    pub(super) fn get_layout(&mut self) -> GuiLayout {
        ::std::mem::take(&mut self.inner.builder.layout)
    }

    pub(super) fn get_node(&mut self) -> GuiNode {
        ::std::mem::take(&mut self.inner.builder.node)
    }

    pub(super) fn set_clipping(&mut self) {
        self.inner.builder.node = GuiNode::clip();
    }

    pub(super) fn push(&mut self, data: GuiComponentData) -> usize {
        self.update_parent_children_count(0);
        let node = self.get_node();
        let layout = self.get_layout();
        let index = self.inner.components.push(node, layout, data);
        self.register_animation(index);
        index
    }

    pub(super) fn push_parent(&mut self, data: GuiComponentData) -> usize {
        let node = self.get_node();
        let layout = self.get_layout();
        let index = self.inner.components.push(node, layout, data);
        self.register_animation(index);
        self.push_stack();
        index
    }

    pub(super) fn pop_parent(&mut self, component_index: usize) {
        let items_params = self.pop_stack();
        let components = &mut self.inner.components;
        components.get_node_mut(component_index).children_count = items_params.children_count;
        components.get_node_mut(component_index).descendants_count = items_params.descendants_count;
        self.update_parent_children_count(items_params.descendants_count);
    }

    fn push_stack(&mut self) {
        self.inner.builder.stack.push(GuiBuilderStack { 
            children_count: 0,
            descendants_count: 0
        })
    }

    fn pop_stack(&mut self) -> GuiBuilderStack {
        match self.inner.builder.stack.pop() {
            Some(stack) => stack,
            _ => unsafe { std::hint::unreachable_unchecked() }
        }
    }

    fn update_parent_children_count(&mut self, descendants_count: u32) {
        self.inner.builder.update_parent_children_count(descendants_count);
    }

    fn register_animation(&mut self, component_index: usize) {
        let after_render = &mut self.inner.after_render;
        let builder = &mut self.inner.builder;

        let animation_data = match builder.next_animation.take() {
            Some(animation) => animation,
            None => { return; }
        };

        let animation_state = GuiAnimationPlayState::new(component_index as u32, animation_data);
        
        if let Some(animation_control_state) = builder.next_animation_control.take() {
            let after_render_index = after_render.len();
            self.inner.state_alloc.insert_animation_listener(animation_control_state, after_render_index);
        }

        after_render.push(AfterRenderHook::UpdateAnimation(animation_state));
    }

}

impl Default for GuiBuilderData {
    fn default() -> Self {
        GuiBuilderData { 
            stack: Vec::with_capacity(8),
            layout: GuiLayout::default(),
            node: GuiNode::default(),
            next_animation: None,
            next_animation_control: None,
        }
    }
}
