use hecs::Entity;
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};
use crate::shared::{PositionF32, AABB, pos, aabb, size};

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

#[derive(Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct BaseSprite {
    pub position: PositionF32,
    pub texcoord: AABB,
    pub highlight_color: [u8; 3],
    pub flags: BaseSpriteFlags,
}

impl BaseSprite {

    pub const fn from_position_static(position: PositionF32, static_sprite: StaticSprite) -> Self {
        BaseSprite {
            position,
            texcoord: static_sprite.texcoord,
            highlight_color: [0, 0, 0],
            flags: BaseSpriteFlags::empty(),
        }
    }

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
    pub fn current_frame(&self) -> StaticSprite {
        StaticSprite {
            texcoord: aabb(pos(self.x + (self.width * (self.current_frame as f32)), self.y), size(self.width, self.height))
        } 
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

#[derive(Copy, Clone)]
pub struct OrderedSprite {
    pub e: Entity,
    pub y: f32,
    pub sprite: BaseSprite,
}

