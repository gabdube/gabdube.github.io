use crate::shared::{pos, size, Scissor, SizeF32};
use super::components::{GuiComponentView, GuiComponentData};
use super::layout::{
    GuiLayoutAlignSelf, GuiLayoutAlignItems, FlexboxItemsLayout, FlexJustifyContent, GuiLayoutSize, GuiLayoutAlignSelfValue
};
use super::Gui;

pub(super) fn layout_compute(gui: &mut Gui) {
    fit_sizing_pass(gui);
    grow_sizing_pass(gui);
    position_pass(gui);
}

//
// Fit Sizing
//

struct LayoutSizingParentFit {
    /// Total size of the items using the parent for alignment
    pub items_size: SizeF32,
    /// Total size of the items that opt-out of the parent layout
    pub items_optout_size: SizeF32,
    pub items_layout: GuiLayoutAlignItems,
}

fn fit_sizing_pass(gui: &mut Gui) {
    let mut parent = LayoutSizingParentFit {
        items_size: size(0.0, 0.0),
        items_optout_size: size(0.0, 0.0),
        items_layout: GuiLayoutAlignItems::NoChildren,
    };

    let mut index = 0;
    while index < gui.components.len() {
        fit_layout_size(gui, &mut index, &mut parent);
    }
}

/// Fetch the default size defined by the component data
fn fit_component_from_default(gui: &Gui, index: usize, view: &mut GuiComponentView) {
    let component_data = gui.components.get_data(index);
    let size = match component_data {
        GuiComponentData::Group |
        GuiComponentData::ListViewBase |
        GuiComponentData::SolidColorBlock(_) |
        GuiComponentData::ScrollView(_) => size(0.0, 0.0),
        GuiComponentData::Borders(borders) => size(borders.border_width * 2.0, borders.border_width * 2.0), 
        GuiComponentData::Spacer(spacer) => size(spacer.width, spacer.height),
        GuiComponentData::Image(image) => image.sprite.texcoord.size(),
        GuiComponentData::Label(label) => label.text.size,
        GuiComponentData::Button(button) => button.minimum_size(),
        GuiComponentData::TextInput(text_input) => text_input.minimum_size(),
        GuiComponentData::ListViewItem(item) => item.minimum_size(),
        GuiComponentData::ScrollbarVertical(bar) => bar.minimum_size(),
        GuiComponentData::Window(window) => window.minimum_size(),
        GuiComponentData::WindowTitleBar(bar) => bar.minimum_size(),
    };

    view.size = size;
}

fn fit_component_from_layout(view: &mut GuiComponentView, children: &LayoutSizingParentFit, align_self: &GuiLayoutAlignSelf) {
    let max_item_size = children.items_size.max(children.items_optout_size);

    view.items_size = max_item_size;

    let default_width = f32::max(view.size.width, view.items_size.width);
    let default_height = f32::max(view.size.height, view.items_size.height);

    view.size.width = match align_self.width {
        GuiLayoutSize::Default | GuiLayoutSize::Grow => default_width,
        GuiLayoutSize::Fixed(value) => value,
        GuiLayoutSize::Min(min_value) => f32::max(default_width, min_value)
    };

    view.size.height = match align_self.height {
        GuiLayoutSize::Default | GuiLayoutSize::Grow => default_height,
        GuiLayoutSize::Fixed(value) => value,
        GuiLayoutSize::Min(min_value) => f32::max(default_height, min_value)
    };
}

fn update_parent_layout_flexbox(parent: &mut LayoutSizingParentFit, view: &GuiComponentView, layout: FlexboxItemsLayout) {
    use super::layout::FlexDirection;
    let [mut size_width, mut size_height] = parent.items_size.splat();
    match layout.direction {
        FlexDirection::Column => {
            size_width = f32::max(size_width, view.size.width);
            size_height += view.size.height;
        },
        FlexDirection::Row => {
            size_width += view.size.width;
            size_height = f32::max(size_height, view.size.height);
        }
    }

    parent.items_size = size(size_width, size_height);
}

fn update_parent_size(parent: &mut LayoutSizingParentFit, view: &GuiComponentView, layout: &GuiLayoutAlignSelf) {
    match layout.align {
        GuiLayoutAlignSelfValue::Parent => {
            match parent.items_layout {
                GuiLayoutAlignItems::NoChildren => { parent.items_size = parent.items_size.max(view.size); },
                GuiLayoutAlignItems::Flexbox(flex) => update_parent_layout_flexbox(parent, view, flex),
            }
        },
        _ => {
            parent.items_optout_size = parent.items_optout_size.max(view.size);
        }
    }
}

fn fit_layout_size(gui: &mut Gui, index: &mut usize, parent: &mut LayoutSizingParentFit) {
    let current = *index;
    *index += 1;

    let node = gui.components.copy_node(current);
    let layout = gui.components.copy_layout(current);
    let mut view = gui.components.copy_view(current);

    fit_component_from_default(gui, current, &mut view);

    let mut child_sizing = LayoutSizingParentFit {
        items_size: size(0.0, 0.0),
        items_optout_size: size(0.0, 0.0),
        items_layout: layout.align_items,
    };

    for _ in 0..node.children_count {
        fit_layout_size(gui, index, &mut child_sizing)
    }

    
    fit_component_from_layout(&mut view, &child_sizing, &layout.align_self);

    update_parent_size(parent, &view, &layout.align_self);
    gui.components.set_view(current, view);
}

//
// Grow sizing
//

struct LayoutSizingParentGrow {
    items_layout: GuiLayoutAlignItems,
    size: SizeF32,
}

fn grow_sizing_pass(gui: &mut Gui) {
    let parent = LayoutSizingParentGrow {
        items_layout: GuiLayoutAlignItems::NoChildren,
        size: gui.view_size,
    };

    let mut index = 0;
    while index < gui.components.len() {
        grow_layout_size(gui, &mut index, &parent);
    }
}

fn grow_parent_layout_flexbox(parent: &LayoutSizingParentGrow, view: &mut GuiComponentView, layout: FlexboxItemsLayout) {
    use super::layout::FlexAlignItems;
    match layout.align_items {
        FlexAlignItems::Stretch => {
            view.size.width = parent.size.width;
        },
        _ => {},
    }
}

fn grow_component_from_layout(parent: &LayoutSizingParentGrow, view: &mut GuiComponentView) {
    match parent.items_layout {
        GuiLayoutAlignItems::NoChildren => {},
        GuiLayoutAlignItems::Flexbox(flex) => grow_parent_layout_flexbox(parent, view, flex),
    }
}

fn grow_layout_size(gui: &mut Gui, index: &mut usize, parent: &LayoutSizingParentGrow) {
    let current = *index;
    *index += 1;

    let node = gui.components.copy_node(current);
    let layout = gui.components.copy_layout(current);
    let mut view = gui.components.copy_view(current);

    // Grow self
    if let GuiLayoutSize::Grow = layout.align_self.width {
        view.size.width = parent.size.width;
    }

    if let GuiLayoutSize::Grow = layout.align_self.height {
        view.size.height = parent.size.height;
    }

    // Grow from parent layout
    if let GuiLayoutAlignSelfValue::Parent = layout.align_self.align {
        grow_component_from_layout(parent, &mut view)
    }

    gui.components.set_view(current, view);

    if node.children_count == 0 {
        return;
    }

    let child_sizing = LayoutSizingParentGrow {
        items_layout: layout.align_items,
        size: view.size,
    };
    for _ in 0..node.children_count {
        grow_layout_size(gui, index, &child_sizing)
    }
}

//
// Positioning
//

struct LayoutPositionParent {
    pub view: GuiComponentView,
    pub items_layout: GuiLayoutAlignItems,
    pub vars: [f32; 1], // Free values that can be used by layout algorithm
}

fn position_pass(gui: &mut Gui) {
    let mut parent = LayoutPositionParent {
        view: GuiComponentView { 
            position: pos(0.0, 0.0),
            size: gui.view_size,
            items_size: size(0.0, 0.0),
            scissor: gui.base_scissor(),
        },
        items_layout: GuiLayoutAlignItems::NoChildren,
        vars: [0.0; 1],
    };

    let mut index = 0;
    while index < gui.components.len() {
        position_layout(gui, &mut index, &mut parent);
    }
}

fn position_layout_parent_flexbox(view: &mut GuiComponentView, parent: &mut LayoutPositionParent, layout: FlexboxItemsLayout) {
    use super::layout::{FlexDirection, FlexAlignItems};
    
    fn cross_axis_offset(parent: f32, item: f32, align: FlexAlignItems) -> f32 {
        match align {
            FlexAlignItems::Start | FlexAlignItems::Stretch => 0.0,
            FlexAlignItems::Center => (parent - item) / 2.0
        }
    }

    let parent_offset = parent.view.position + layout.children_offset;
    let parent_size = parent.view.size;
    let mut offset = parent.vars[0];

    let justify_offset = match layout.justify_content {
        FlexJustifyContent::Start => 0.0,
        FlexJustifyContent::Center => match layout.direction {
            FlexDirection::Column => (parent.view.size.height - parent.view.items_size.height) / 2.0,
            FlexDirection::Row =>  (parent.view.size.width - parent.view.items_size.width) / 2.0,
        }
    };

    match layout.direction {
        FlexDirection::Column => {
            view.position.x = parent_offset.x + cross_axis_offset(parent_size.width, view.size.width, layout.align_items);
            view.position.y = parent_offset.y + justify_offset + offset;
            offset += view.size.height;
        },
        FlexDirection::Row => {
            view.position.x = parent_offset.x + justify_offset + offset;
            view.position.y = parent_offset.y + cross_axis_offset(parent_size.height, view.size.height, layout.align_items);
            offset += view.size.width;
        }
    }

    parent.vars[0] = offset;
}

fn position_layout_parent(view: &mut GuiComponentView, parent: &mut LayoutPositionParent) {
    match parent.items_layout {
        GuiLayoutAlignItems::NoChildren => {
            let parent_pos = parent.view.position;
            view.position.x = parent_pos.x;
            view.position.y = parent_pos.y;
        },
        GuiLayoutAlignItems::Flexbox(flex) => {
            position_layout_parent_flexbox(view, parent, flex);
        }
    }
}

fn position_layout(gui: &mut Gui, index: &mut usize, parent: &mut LayoutPositionParent) {
    let current = *index;
    *index += 1;

    let node = gui.components.copy_node(current);
    let layout = gui.components.copy_layout(current);
    let mut view = gui.components.copy_view(current);

    let parent_pos = parent.view.position;
    match layout.align_self.align {
        GuiLayoutAlignSelfValue::Parent => {
            position_layout_parent(&mut view, parent);
        }
        GuiLayoutAlignSelfValue::TopLeft => {
            view.position.x = parent_pos.x;
            view.position.y = parent_pos.y;
        },
        GuiLayoutAlignSelfValue::TopRight => {
            let parent_width = parent.view.size.width;
            view.position.x = (parent_pos.x + parent_width) - view.size.width;
            view.position.y = parent_pos.y;
        },
        GuiLayoutAlignSelfValue::Center => {
            let parent_size = parent.view.size;
            view.position.x = parent_pos.x + (parent_size.width - view.size.width) / 2.0;
            view.position.y = parent_pos.y + (parent_size.height - view.size.height) / 2.0;
        }
    }

    view.position += layout.align_self.offset;

    view.scissor = match [layout.visible, node.clip == 1] {
        [false, _] => Scissor::default(),
        [true, false] => parent.view.scissor,
        [true, true] => {
            Scissor::from_position_and_size(view.position, view.size)
                .clip(parent.view.scissor)
        }
    };  

    gui.components.set_view(current, view);
    
    if node.children_count == 0 {
        return;
    }
    
    let mut parent = LayoutPositionParent { 
        view,
        items_layout: layout.align_items,
        vars: [0.0; 1]
    };
    for _ in 0..node.children_count {
        position_layout(gui, index, &mut parent);
    }
}

