use zerocopy_derive::{FromBytes, Immutable, IntoBytes, TryFromBytes};

macro_rules! flags {
    ($get:ident, $value:expr) => {
        #[inline(always)] pub const fn $get(&self) -> bool { self.0 & $value > 0 }
    };

    ($get:ident, $set:ident, $value:expr) => {
        #[inline(always)] pub fn $set(&mut self) { self.0 |= $value; }
        #[inline(always)] pub const fn $get(&self) -> bool { self.0 & $value > 0 }
    };

    ($get:ident, $set:ident, $clear:ident, $value:expr) => {
        #[inline(always)] pub fn $set(&mut self) { self.0 |= $value; }
        #[inline(always)] pub fn $clear(&mut self) { self.0 &= !$value; }
        #[inline(always)] pub const fn $get(&self) -> bool { self.0 & $value > 0 }
    };
}

#[derive(Default, Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct RenderFlags(pub u32);

impl RenderFlags {
    pub const UPDATE_TERRAIN: u32 = 0x1;
    pub const UPDATE_VIEW_OFFSET: u32 = 0x2;
    pub const UPDATE_ZOOM: u32 = 0x4;
    pub const UPDATE_ANIMATIONS: u32 = 0x8;

    flags!(update_terrain, set_update_terrain, Self::UPDATE_TERRAIN);
    flags!(update_view_offset, set_update_view_offset, Self::UPDATE_VIEW_OFFSET);
    flags!(update_zoom, set_update_zoom, Self::UPDATE_ZOOM);
    flags!(update_animations, set_update_animations, Self::UPDATE_ANIMATIONS);

    pub fn clear(&mut self) { self.0 = 0; }
}

#[derive(Default, Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct BaseSpriteFlags(pub u8);

impl BaseSpriteFlags {
    pub const FLIPPED: u8  = 0x1;
    pub const HIGHLIGHTED: u8 = 0x2;

    flags!(flipped, set_flipped, clear_flipped, Self::FLIPPED);
    flags!(highlighted, set_highlighted, clear_highlighted, Self::HIGHLIGHTED);

    #[inline(always)]
    pub const fn empty() -> Self {
        BaseSpriteFlags(0)
    }
}

#[derive(Copy, Clone, PartialEq, Default, IntoBytes, TryFromBytes, Immutable)]
#[repr(u8)]
pub enum ButtonState {
    #[default]
    Released = 0,
    JustReleased = 1,
    Pressed = 2,
    JustPressed = 3,
}

impl ButtonState {
    pub fn flip(&mut self) {
        match self {
            Self::JustPressed => { *self = Self::Pressed; }
            Self::JustReleased => { *self = Self::Released; }
            _ => {}
        }
    }

    pub fn just_released(self) -> bool { self == Self::JustReleased }
    pub fn released(self) -> bool { self == Self::JustReleased || self == Self::Released }
    pub fn just_pressed(self) -> bool { self == Self::JustPressed }
    pub fn pressed(self) -> bool { self == Self::JustPressed || self == Self::Pressed }
}

