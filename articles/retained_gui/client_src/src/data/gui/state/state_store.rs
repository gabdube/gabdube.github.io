use std::marker::PhantomData;
use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::{data::gui::GuiAnimationControl, shared::PositionF32};
use super::super::components::{GuiImageStyle};

pub(super) const MAX_LISTENERS_COUNT: usize = 8;

#[repr(transparent)]
#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct ChildrenOffsetY(pub f32);

#[repr(transparent)]
#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct ChildrenOffsetX(pub f32);

#[repr(transparent)]
#[derive(Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct LayoutOffset(pub PositionF32);

pub struct GuiState<T> {
    pub(super) index: u32,
    pub(super) type_id: u16,
    pub(super) generation: u16,
    pub(super) _p: PhantomData<T>
}

pub enum GuiStateStore {
    Bool(bool),
    Usize(usize),
    Image(GuiImageStyle),
    String(String),
    ChildrenOffsetY(ChildrenOffsetY),
    ChildrenOffsetX(ChildrenOffsetX),
    LayoutOffset(LayoutOffset),
    AnimationControl(GuiAnimationControl),
}

impl GuiStateStore {
    pub fn is_animation(&self) -> bool {
        match self {
            Self::AnimationControl(_) => true,
            _ => false
        }
    }

    pub fn type_id(&self) -> u16 {
        match self {
            Self::Bool(_) => 1,
            Self::Usize(_) => 2,
            Self::Image(_) => 3,
            Self::String(_) => 4,
            Self::ChildrenOffsetY(_) => 5,
            Self::ChildrenOffsetX(_) => 6,
            Self::LayoutOffset(_) => 7,
            Self::AnimationControl(_) => 8,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "Bool",
            Self::Usize(_) => "Usize",
            Self::Image(_) => "Image",
            Self::String(_) => "String",
            Self::ChildrenOffsetY(_) => "ChildrenOffsetY",
            Self::ChildrenOffsetX(_) => "ChildrenOffsetX",
            Self::LayoutOffset(_) => "LayoutOffset",
            Self::AnimationControl(_) => "GuiAnimationControl",
        }
    }
}

#[derive(Debug, Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct ListenerIndex {
    pub flags: u32,
    pub value: u32,
}

impl ListenerIndex {
    const EMPTY_FLAG: u32     = 0b0001;
    const LAYOUT_FLAG: u32    = 0b0010;
    const COMPONENT_FLAG: u32 = 0b0100;
    const ANIMATION_FLAG: u32 = 0b1000;
    pub const MAX_INDEX: usize = u32::MAX as usize;

    pub const fn new() -> Self {
        ListenerIndex { flags: Self::EMPTY_FLAG, value: 0 }
    }

    pub const fn new_component(value: usize) -> Self {
        ListenerIndex { flags: Self::COMPONENT_FLAG, value: value as u32 }
    }

    pub const fn new_layout(value: usize) -> Self {
        ListenerIndex { flags: Self::LAYOUT_FLAG, value: value as u32 }
    }

    pub const fn new_animation(value: usize) -> Self {
        ListenerIndex { flags: Self::ANIMATION_FLAG, value: value as u32 }
    }

    pub fn index(&self) -> usize {
        self.value as usize
    }

    pub fn is_layout(&self) -> bool {
        self.flags & Self::LAYOUT_FLAG > 0
    }

    pub fn is_empty(&self) -> bool {
        self.flags & Self::EMPTY_FLAG > 0
    }
}

#[derive(Debug, Copy, Clone, Immutable, IntoBytes, FromBytes)]
pub struct Listeners([ListenerIndex; MAX_LISTENERS_COUNT]);

impl Listeners {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn find_empty(&self) -> Option<usize> {
        self.0.iter().position(|value| value.is_empty() )
    }

    pub fn with_values(&self) -> impl Iterator<Item=ListenerIndex> {
        let mut index = 0;
        return ::std::iter::from_fn(move || {
            let listeners_index = self.0.get(index).copied()?;
            if listeners_index.is_empty() {
                return None;
            }

            index += 1;
            Some(listeners_index)
        });
    }

    pub fn set(&mut self, index: usize, value: ListenerIndex) {
        self.0[index] = value;
    }
}

pub struct GuiStateStoreWrapper {
    pub store: GuiStateStore,
    pub listeners: Listeners,
}


//
// Other impls
//

impl<T> Copy for GuiState<T> {}
impl<T> Clone for GuiState<T> {
    fn clone(&self) -> Self {
        GuiState {
            index: self.index,
            type_id: self.type_id,
            generation: self.generation,
            _p: self._p
        }
    }
}

impl<T> Default for GuiState<T> {
    fn default() -> Self {
        GuiState {
            index: 0,
            type_id: 0,
            generation: 0,
            _p: PhantomData
        }
    }
}

impl Default for Listeners {
    fn default() -> Self {
        Listeners([ListenerIndex::new(); MAX_LISTENERS_COUNT])
    }
}

impl<T> crate::store::StoreLoad for GuiState<T> {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&(self.index as u32));
        writer.write(&(self.type_id as u32));
        writer.write(&(self.generation as u32));
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let state = GuiState {
            index: reader.try_read::<u32>()?,
            type_id: reader.try_read::<u32>()? as u16,
            generation: reader.try_read::<u32>()? as u16,
            _p: PhantomData
        };
    
        Ok(state)
    }
}

impl crate::store::StoreLoad for GuiStateStore {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        let id = self.type_id() as u32;
        writer.write(&id);

        match self {
            Self::Bool(value) => {
                writer.write_bool(*value);
            }
            Self::Usize(value) => {
                writer.write(value);
            }
            Self::Image(image) => {
                writer.write(image);
            },
            Self::String(value) => {
                writer.write_str(value);
            },
            Self::ChildrenOffsetY(value) => {
                writer.write(value);
            },
            Self::ChildrenOffsetX(value) => {
                writer.write(value);
            },
            Self::LayoutOffset(value) => {
                writer.write(value);
            },
            Self::AnimationControl(value) => {
                writer.write(value);
            }
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let id: u32 = reader.try_read()?;
        match id {
            1 => Ok(GuiStateStore::Bool(reader.try_read_bool()?)),
            2 => Ok(GuiStateStore::Usize(reader.try_read()?)),
            3 => Ok(GuiStateStore::Image(reader.try_read()?)),
            4 => Ok(GuiStateStore::String(reader.read_str().to_string())),
            5 => Ok(GuiStateStore::ChildrenOffsetY(reader.try_read()?)),
            6 => Ok(GuiStateStore::ChildrenOffsetX(reader.try_read()?)),
            7 => Ok(GuiStateStore::LayoutOffset(reader.try_read()?)),
            8 => Ok(GuiStateStore::AnimationControl(reader.try_read()?)),
            _ => Err(assets_err!("Unknown identifer {id} for GuiStateStore"))
        }
    }
}

impl crate::store::StoreLoad for GuiStateStoreWrapper {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.listeners);
        self.store.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let listeners = reader.try_read()?;
        let store = GuiStateStore::load(reader)?;
        Ok(GuiStateStoreWrapper { store, listeners })
    }
}
