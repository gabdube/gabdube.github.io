use zerocopy_derive::{FromBytes, Immutable, IntoBytes, TryFromBytes};

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
pub struct DebugFlags(pub u32);

impl DebugFlags {
    pub const DEBUG_WORLD_GRID: u32 = 0x1;
    pub const DEBUG_DISPLAY_GRID: u32 = 0x2;

    flags!(debug_world_grid, Self::DEBUG_WORLD_GRID);
    flags!(debug_display_grid, Self::DEBUG_DISPLAY_GRID);

    pub fn toggle(&mut self, flag: u32) {
        self.0 ^= flag;
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

pub mod keys {
    use ::std::num::NonZeroU8;

    #[derive(Copy, Clone, Hash, PartialEq, Eq)]
    pub struct Key(::std::num::NonZeroU8);

    pub const KEY_DIGIT_1: Key = Key::from_const(1);
    pub const KEY_DIGIT_2: Key = Key::from_const(2);

    #[allow(dead_code)]
    pub const KEY_INVALID: Key = Key::from_const(255);

    impl Key {
        pub fn from_str(value: &str) -> Option<Self> {
            match value {
                "Digit1" => Some(KEY_DIGIT_1),
                "Digit2" => Some(KEY_DIGIT_2),
                _ => None,
            }
        }

        pub const fn from_const(value: u8) -> Self {
            unsafe {
                match value {
                    0 => Key(NonZeroU8::new_unchecked(255)),
                    value => Key(NonZeroU8::new_unchecked(value)),
                }
            }
        }
    }
}
