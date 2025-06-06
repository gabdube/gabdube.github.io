use zerocopy_derive::{FromBytes, Immutable, IntoBytes, TryFromBytes};
use crate::shared::{PositionF32, AABB, pos, size, aabb};

macro_rules! flags {
    ($get:ident, $set:ident, $clear:ident, $value:expr) => {
        #[inline(always)] pub fn $set(&mut self) { self.0 |= $value; }
        #[inline(always)] pub fn $clear(&mut self) { self.0 &= !$value; }
        #[inline(always)] pub const fn $get(&self) -> bool { self.0 & $value > 0 }
    };
}

#[derive(Default, Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct GameFlags(pub u32);

impl GameFlags {
    pub const UPDATE_ANIMATIONS: u32 = 0b0001;
    pub const UPDATE_TERRAIN: u32 = 0b0010;
    pub const UPDATE_GUI: u32 = 0b0100;
    pub const UPDATE_VIEW_OFFSET: u32 = 0b1000;

    flags!(update_animations, set_update_animations, clear_update_animations, Self::UPDATE_ANIMATIONS);
    flags!(update_terrain, set_update_terrain, clear_update_terrain, Self::UPDATE_TERRAIN);
    flags!(update_gui, set_update_gui, clear_update_gui, Self::UPDATE_GUI);
    flags!(update_view_offset, set_update_view_offset, clear_update_view_offset, Self::UPDATE_VIEW_OFFSET);
}

#[derive(Default, Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct DebugFlags(pub u32);

impl DebugFlags {
    pub const SHOW_NAVMESH: u32 = 0x1;
    pub const SHOW_COLLISION_BOXES: u32 = 0x2;
    pub const SHOW_HOVERED_TRIANGLE: u32 = 0x4;
    pub const SHOW_CELL_CENTERS: u32 = 0x8;
    pub const SHOW_PATH: u32 = 0x10;
    pub const SHOW_BLOCKED_CELLS: u32 = 0x20;
}

#[derive(Default, Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct BaseSpriteFlags(pub u32);

impl BaseSpriteFlags {
    #[inline(always)]
    pub fn empty() -> Self { BaseSpriteFlags(0) }

    #[inline(always)]
    pub fn value(&self) -> i32 { self.0 as i32 }
}

#[derive(Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct BaseSprite {
    pub position: PositionF32,
    pub texcoord: AABB,
    pub flags: BaseSpriteFlags,
}

impl BaseSprite {
    pub fn rect(&self) -> AABB {
        aabb(self.position, self.texcoord.size())
    }
}

#[derive(Default, Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct AnimationState {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub current_frame: u16,
    pub max_frame: u16,
}

impl AnimationState {
    pub fn current_frame(&self) -> AABB {
        aabb(pos(self.x + (self.width * (self.current_frame as f32)), self.y), size(self.width, self.height))
    }
}

#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
pub struct StaticSprite {
    pub texcoord: AABB,
}

#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
pub struct AnimatedSprite {
    pub sprite_base: AABB,
    pub frame_count: u32,
}

impl AnimatedSprite {
    pub fn sprite(&self) -> StaticSprite {
        let [width, _] = self.sprite_base.splat_size();
        StaticSprite {
            texcoord: AABB { 
                left: self.sprite_base.left,
                top: self.sprite_base.top,
                right: self.sprite_base.left + (width / self.frame_count as f32),
                bottom: self.sprite_base.bottom
            }
        }
    }

    pub fn animate(&self) -> AnimationState {
        let [mut width, height] = self.sprite_base.splat_size();
        width /= self.frame_count as f32; 
        AnimationState { 
            x: self.sprite_base.left,
            y: self.sprite_base.top,
            width,
            height,
            current_frame: 0,
            max_frame: self.frame_count as u16
        }
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

    pub fn released(self) -> bool { self == Self::JustReleased || self == Self::Released }
    pub fn just_pressed(self) -> bool { self == Self::JustPressed }
}
