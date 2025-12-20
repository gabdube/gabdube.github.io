use crate::data::gui::components::{
    GuiNode, GuiComponentData, GuiListViewItemStyles
};
use crate::data::gui::layout::{GuiLayout, GuiLayoutAlignItems, GuiLayoutAlignSelf};
use crate::data::gui::state::GuiState;
use crate::data::gui::{Gui, GuiInternalEvent};

pub struct GuiListViewBuilder<'a> {
    pub(super) inner: &'a mut Gui,
    pub(super) styles: GuiListViewItemStyles,
    pub(super) item_height: f32,
    pub(super) text_size: f32,
    pub(super) on_click: GuiInternalEvent,
    pub(super) selected_state: GuiState<usize>,
}

impl<'a> GuiListViewBuilder<'a> {
    pub(super) fn push_item(&mut self, data: GuiComponentData) {
        self.update_parent_children_count();

        let layout = Self::item_layout();
        self.inner.components.push(
            GuiNode::default(),
            layout,
            data,
        );
    }

    fn update_parent_children_count(&mut self) {
        self.inner.builder.update_parent_children_count(0);
    }

    const fn item_layout() -> GuiLayout {
        GuiLayout {
            align_self: GuiLayoutAlignSelf::parent_grow_width(),
            align_items: GuiLayoutAlignItems::NoChildren,
            visible: true,
        }
    }

}
