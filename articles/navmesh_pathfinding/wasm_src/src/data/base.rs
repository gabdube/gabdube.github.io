use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
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
    pub const UPDATE_ANIMATIONS: u32 = 0b001;

    flags!(update_animations, set_update_animations, clear_update_animations, Self::UPDATE_ANIMATIONS);
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

#[derive(Copy, Clone, FromBytes, IntoBytes, Immutable)]
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
pub struct AnimatedSprite {
    pub sprite_base: AABB,
    pub frame_count: u32,
}

impl AnimatedSprite {
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


