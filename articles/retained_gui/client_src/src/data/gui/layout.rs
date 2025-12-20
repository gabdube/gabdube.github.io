use crate::shared::{PositionF32, SizeF32, pos};
use super::state::{GuiStateStore, ChildrenOffsetY, ChildrenOffsetX, LayoutOffset};

#[derive(Copy, Clone)]
pub enum FlexDirection {
    Column,
    Row
}

#[derive(Copy, Clone)]
pub enum FlexAlignItems {
    Start,
    Center,
    Stretch
}

#[derive(Copy, Clone)]
pub enum FlexJustifyContent {
    Start,
    Center,
}

#[derive(Copy, Clone)]
pub struct FlexboxItemsLayout {
    pub children_offset: PositionF32,
    pub direction: FlexDirection,
    pub align_items: FlexAlignItems,
    pub justify_content: FlexJustifyContent,
}

#[derive(Copy, Clone)]
pub enum GuiLayoutAlignSelfValue {
    /// Use parent "align_items" to size & position component
    Parent,
    /// Component will be centered in the parent
    Center,
    /// Component will be placed at the top-left corner of the parent
    TopLeft,
    /// Component will be placed at the top-right corner of the parent
    TopRight,
}

#[derive(Copy, Clone)]
pub enum GuiLayoutSize {
    Default,
    Grow,
    Min(f32),
    Fixed(f32),
}

#[derive(Copy, Clone)]
pub struct GuiLayoutAlignSelf {
    /// Tell the component how to align itself
    pub align: GuiLayoutAlignSelfValue,
    /// Add an offset to the component final position
    pub offset: PositionF32,
    /// Optionally overrides the component width
    pub width: GuiLayoutSize,
    /// Optionally overrides the component height
    pub height: GuiLayoutSize,
}

impl GuiLayoutAlignSelf {

    #[inline(always)]
    pub const fn parent() -> Self {
        GuiLayoutAlignSelf {
            align: GuiLayoutAlignSelfValue::Parent,
            offset: pos(0.0, 0.0),
            width: GuiLayoutSize::Default,
            height: GuiLayoutSize::Default,
        }
    }

    #[inline(always)]
    pub const fn parent_grow_width() -> Self {
        GuiLayoutAlignSelf {
            align: GuiLayoutAlignSelfValue::Parent,
            offset: pos(0.0, 0.0),
            width: GuiLayoutSize::Grow,
            height: GuiLayoutSize::Default,
        }
    }

    #[inline(always)]
    pub const fn parent_fixed_width(width: f32) -> Self {
        GuiLayoutAlignSelf {
            align: GuiLayoutAlignSelfValue::Parent,
            offset: pos(0.0, 0.0),
            width: GuiLayoutSize::Fixed(width),
            height: GuiLayoutSize::Default,
        }
    }

    #[inline(always)]
    pub const fn parent_fixed_size(size: SizeF32) -> Self {
        GuiLayoutAlignSelf {
            align: GuiLayoutAlignSelfValue::Parent,
            offset: pos(0.0, 0.0),
            width: GuiLayoutSize::Fixed(size.width),
            height: GuiLayoutSize::Fixed(size.height),
        }
    }

    #[inline(always)]
    pub const fn center() -> Self {
        GuiLayoutAlignSelf {
            align: GuiLayoutAlignSelfValue::Center,
            offset: pos(0.0, 0.0),
            width: GuiLayoutSize::Default,
            height: GuiLayoutSize::Default,
        }
    }

    #[inline(always)]
    pub const fn center_min_size(size: SizeF32) -> Self {
        GuiLayoutAlignSelf {
            align: GuiLayoutAlignSelfValue::Center,
            offset: pos(0.0, 0.0),
            width: GuiLayoutSize::Min(size.width),
            height: GuiLayoutSize::Min(size.height),
        }
    }

    #[inline(always)]
    pub const fn background() -> Self {
        GuiLayoutAlignSelf {
            align: GuiLayoutAlignSelfValue::TopLeft,
            offset: pos(0.0, 0.0),
            width: GuiLayoutSize::Grow,
            height: GuiLayoutSize::Grow,
        }
    }

    #[inline(always)]
    pub const fn scrollbar_vertical() -> Self {
        GuiLayoutAlignSelf {
            align: GuiLayoutAlignSelfValue::TopRight,
            offset: pos(0.0, 0.0),
            width: GuiLayoutSize::Default,
            height: GuiLayoutSize::Grow,
        }
    }
    
}


#[derive(Copy, Clone)]
pub enum GuiLayoutAlignItems {
    NoChildren,
    Flexbox(FlexboxItemsLayout),
}

 
#[derive(Copy, Clone)]
pub struct GuiLayout {
    pub align_self: GuiLayoutAlignSelf,
    pub align_items: GuiLayoutAlignItems,
    pub visible: bool,
}

impl GuiLayout {

    pub(super) fn sync_state_data(&mut self, data: &GuiStateStore) {
        match data {
            GuiStateStore::Bool(value) => { self.visible = *value; }
            GuiStateStore::ChildrenOffsetY(ChildrenOffsetY(offset)) => { self.set_children_offset_y(*offset); },
            GuiStateStore::ChildrenOffsetX(ChildrenOffsetX(offset)) => { self.set_children_offset_x(*offset); },
            GuiStateStore::LayoutOffset(LayoutOffset(offset)) => { self.align_self.offset = *offset; },
            _ => {},
        }
    }

    fn set_children_offset_x(&mut self, offset: f32) {
        match &mut self.align_items {
            GuiLayoutAlignItems::Flexbox(flexbox) => {
                flexbox.children_offset.x = offset;
            },
            GuiLayoutAlignItems::NoChildren => {},
        }
    }

    fn set_children_offset_y(&mut self, offset: f32) {
        match &mut self.align_items {
            GuiLayoutAlignItems::Flexbox(flexbox) => {
                flexbox.children_offset.y = -offset;
            },
            GuiLayoutAlignItems::NoChildren => {},
        }
    }

}


impl crate::store::StoreLoad for GuiLayout {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        self.align_self.align.store(writer);
        self.align_self.width.store(writer);
        self.align_self.height.store(writer);
        writer.write(&self.align_self.offset);

        match self.align_items {
            GuiLayoutAlignItems::NoChildren => { writer.write(&1u32); }
            GuiLayoutAlignItems::Flexbox(flex) => {
                writer.write(&2u32);
                flex.store(writer);
            }
        }

        writer.write_bool(self.visible);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let align = GuiLayoutAlignSelfValue::load(reader)?;
        let width = GuiLayoutSize::load(reader)?;
        let height = GuiLayoutSize::load(reader)?;
        let offset = reader.try_read()?;
        let align_self = GuiLayoutAlignSelf {
            align,
            offset,
            width,
            height,
        };

        let align_items_raw: u32 = reader.try_read()?;
        let align_items = match align_items_raw {
            1 => GuiLayoutAlignItems::NoChildren,
            2 => GuiLayoutAlignItems::Flexbox(FlexboxItemsLayout::load(reader)?),
            _ => { return Err(assets_err!("Unknown identifier {align_items_raw} for GuiLayoutSize")); }
        };

        let visible = reader.try_read_bool()?;

        let layout = GuiLayout {
            align_self,
            align_items,
            visible
        };

        Ok(layout)
    }
}

impl crate::store::StoreLoad for FlexboxItemsLayout {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.children_offset);

        match self.direction {
            FlexDirection::Column => writer.write(&0u32),
            FlexDirection::Row => writer.write(&1u32),
        }

        match self.align_items {
            FlexAlignItems::Start => writer.write(&0u32),
            FlexAlignItems::Center => writer.write(&1u32),
            FlexAlignItems::Stretch => writer.write(&2u32),
        }

        match self.justify_content {
            FlexJustifyContent::Start => writer.write(&0u32),
            FlexJustifyContent::Center => writer.write(&1u32),
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let children_offset = reader.try_read()?;
        let direction_raw: u32 = reader.try_read()?;
        let align_items_raw: u32 = reader.try_read()?;
        let justify_content_raw: u32 = reader.try_read()?;

        let direction = match direction_raw {
            0 => FlexDirection::Column,
            1 => FlexDirection::Row,
            value => { return Err(assets_err!("Unknown flex direction id {value}")); }
        };

        let align_items = match align_items_raw {
            0 => FlexAlignItems::Start,
            1 => FlexAlignItems::Center,
            2 => FlexAlignItems::Stretch,
            value => { return Err(assets_err!("Unknown flex align items id {value}")); }
        };

        let justify_content = match justify_content_raw {
            0 =>  FlexJustifyContent::Start,
            1 =>  FlexJustifyContent::Center,
            value => { return Err(assets_err!("Unknown flex align items id {value}")); }
        };
        

        let flex = FlexboxItemsLayout {
            children_offset,
            direction,
            align_items,
            justify_content,
        };

        Ok(flex)
    }
}

impl crate::store::StoreLoad for GuiLayoutAlignSelfValue {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        let align_value = match self {
            GuiLayoutAlignSelfValue::Parent   => 0u32,
            GuiLayoutAlignSelfValue::Center   => 1,
            GuiLayoutAlignSelfValue::TopLeft  => 2,
            GuiLayoutAlignSelfValue::TopRight => 3
        };

        writer.write(&align_value);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let id: u32 = reader.try_read()?;
        let value = match id {
            0 => GuiLayoutAlignSelfValue::Parent,
            1 => GuiLayoutAlignSelfValue::Center,
            2 => GuiLayoutAlignSelfValue::TopLeft,
            3 => GuiLayoutAlignSelfValue::TopRight,
            _ => { return Err(assets_err!("Unknown identifier {id} for GuiLayoutAlignSelfValue")) }
        };

        Ok(value)
    }
}

impl crate::store::StoreLoad for GuiLayoutSize {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        match self {
            GuiLayoutSize::Default => { 
                writer.write(&0u32);
            },
            GuiLayoutSize::Grow => { 
                writer.write(&1u32);
            },
            GuiLayoutSize::Min(val) => { 
                writer.write(&2u32);
                writer.write(val);
            },
            GuiLayoutSize::Fixed(val) => { 
                writer.write(&3u32);
                writer.write(val);
            },
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let id = reader.try_read::<u32>()?;
        let value = match id {
            0 => GuiLayoutSize::Default,
            1 => GuiLayoutSize::Grow,
            2 => {
                let value = reader.try_read()?;
                GuiLayoutSize::Min(value)
            },
            3 => {
                let value = reader.try_read()?;
                GuiLayoutSize::Fixed(value)
            },
            _ => { return Err(assets_err!("Unknown identifier {id} for GuiLayoutSize")); }
        };

        Ok(value)
    }
}

impl Default for GuiLayoutAlignSelf {
    fn default() -> Self {
        GuiLayoutAlignSelf { 
            align: GuiLayoutAlignSelfValue::Parent,
            offset: pos(0.0, 0.0),
            width: GuiLayoutSize::Default,
            height: GuiLayoutSize::Default,
        }
    }
}

impl Default for GuiLayout {
    fn default() -> Self {
        GuiLayout {
            align_self: GuiLayoutAlignSelf::default(),
            align_items: GuiLayoutAlignItems::NoChildren,
            visible: true
        }
    }
}
