#![allow(dead_code)]

use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use std::ops::{SubAssign, Sub, Add, AddAssign};

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

/// Unique resource owned by the engine
#[derive(Debug, Copy, Clone, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct ExternalId(pub u32);

impl Default for ExternalId {
    fn default() -> Self {
        ExternalId(u32::MAX)
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, FromBytes, IntoBytes, Immutable)]
#[repr(align(4))]
pub struct ColorRGBA8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorRGBA8 {
    pub const fn splat(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}


#[derive(Default, Debug, Copy, Clone, PartialEq, FromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct PositionF32 {
    pub x: f32,
    pub y: f32,
}

impl PositionF32 {
    #[inline(always)]
    pub const fn splat(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    pub fn distance(&self, other: PositionF32) -> f32 {
        let x2 = other.x - self.x;
        let y2 = other.y - self.y;
        f32::sqrt(x2*x2 + y2*y2)
    }

    pub fn roughly_equal(&self, other: PositionF32) -> bool {
        (self.x - other.x).abs() < f32::EPSILON && (self.y - other.y).abs() < f32::EPSILON
    }
}

impl Add for PositionF32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        PositionF32 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl AddAssign for PositionF32 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for PositionF32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        PositionF32 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl SubAssign for PositionF32 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd, FromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct SizeF32 {
    pub width: f32,
    pub height: f32,
}

impl SizeF32 {

    pub fn max(self, other: SizeF32) -> Self {
        size(f32::max(self.width, other.width), f32::max(self.height, other.height))
    }

    pub fn splat(&self) -> [f32; 2] {
        [self.width, self.height]
    }

}

#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd, FromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct SizeU32 {
    pub width: u32,
    pub height: u32,
}

impl SizeU32 {
    pub const fn splat(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default, FromBytes, IntoBytes, Immutable)]
pub struct AABB {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32
}

impl AABB {
    #[inline(always)]
    pub const  fn splat(&self) -> [f32; 4] {
        [self.left, self.top, self.right, self.bottom]
    }

    #[inline(always)]
    pub const fn splat_size(&self) -> [f32; 2] {
        [self.right - self.left, self.bottom - self.top]
    }

    #[inline(always)]
    pub fn position(&self) -> PositionF32 {
        pos(self.left, self.top)
    }

    #[inline(always)]
    pub const fn size(&self) -> SizeF32 {
        SizeF32 { width: self.right - self.left, height: self.bottom - self.top }
    }

    #[inline(always)]
    pub const fn point_inside(&self, point: PositionF32) -> bool {
        point.x >= self.left && point.x <= self.right && point.y >= self.top && point.y <= self.bottom
    }

    #[inline(always)]
    pub const fn intersects(&self, other: &Self) -> bool {
        if self.right < other.left || other.right < self.left {
            return false
        }

        if self.bottom < other.top || other.bottom < self.top {
            return false
        }

        true
    }

    #[inline(always)]
    pub const fn height(&self) -> f32 {
        self.bottom - self.top
    }

}

#[derive(Copy, Clone, PartialEq, Default, FromBytes, IntoBytes, Immutable)]
#[allow(non_camel_case_types)]
pub struct AABB_U32 {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32
}

#[derive(Debug, Copy, Clone, PartialEq, Default, FromBytes, IntoBytes, Immutable)]
pub struct Scissor {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16
}

impl Scissor {
    #[inline(always)]
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Scissor { x, y, width, height }
    }

    #[inline(always)]
    pub fn splat(&self) -> [u16; 4] {
        [self.x, self.y, self.width, self.height]
    }

    #[inline(always)]
    pub fn from_position_and_size(position: PositionF32, size: SizeF32) -> Self {
        Scissor::new(position.x as u16, position.y as u16, size.width as u16, size.height as u16)
    }

    #[inline(always)]
    pub fn is_zero_sized(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn clip(self, other: Self) -> Self {
        let left1 = self.x;
        let right1 = self.x + self.width;
        let left2 = other.x;
        let right2 = other.x + other.width;
        let left = u16::max(left1, left2);
        let right = u16::min(right1, right2);

        let top1 = self.y;
        let bottom1 = self.y + self.height;
        let top2 = other.y;
        let bottom2 = other.y + other.height;
        let top = u16::max(top1, top2);
        let bottom = u16::min(bottom1, bottom2);

        Scissor { x: left, y: top, width: right-left, height: bottom-top }
    }
}

//
// Helpers method
//

pub fn parse_f32(v: Option<&&str>) -> f32 { v.and_then(|&val| str::parse::<f32>(val).ok() ).unwrap_or(0.0) }
pub fn parse_u32(v: Option<&&str>) -> u32 { v.and_then(|&val| str::parse::<u32>(val).ok() ).unwrap_or(0) }

pub const fn rgba8(r: u8, g: u8, b: u8, a: u8) -> ColorRGBA8 {
    ColorRGBA8 { r, g, b, a }
}

pub const fn pos(x: f32, y: f32) -> PositionF32 {
    PositionF32 { x, y }
}

pub const fn size(width: f32, height: f32) -> SizeF32 {
    SizeF32 { width, height }
}

pub const fn size_u32(width: u32, height: u32) -> SizeU32 {
    SizeU32 { width, height }
}

pub const fn aabb(position: PositionF32, size: SizeF32) -> AABB {
    AABB {
        left: position.x,
        top: position.y,
        right: position.x + size.width,
        bottom: position.y + size.height
    }
}

pub const fn aabb_u32(left: u32, top: u32, width: u32, height: u32) -> AABB_U32 {
    AABB_U32 { left, top, right: left+width, bottom: top+height }
}

/// Split a csv string into up to `MAX_ARGS` parameters. Calls `callback` for each line splitted.
pub fn split_csv<const MAX_ARGS: usize, CB: FnMut(&[&str])>(csv: &str, mut callback: CB) {
    let mut start = 0;
    let mut end = 0;
    let last_char_index = csv.len();
    let mut chars_iter = csv.chars();
    let mut args: [&str; MAX_ARGS] = [""; MAX_ARGS];
    while let Some(c) = chars_iter.next() {
        end += 1;
        if c == '\n' || end == last_char_index {
            let line = &csv[start..end];
            let mut args_count = 0;
            for substr in line.split(';') {
                if args_count < MAX_ARGS {
                    args[args_count] = substr;
                    args_count += 1;
                }
            }

            if args_count > 1 {
                callback(&args[0..args_count]);
            }

            start = end;
        }
    }
}

pub fn merge_error(err: &mut Option<crate::error::Error>, new: crate::error::Error) {
    if err.is_none() {
        *err = Some(new);
    } else {
        err.as_mut().unwrap().merge(new);
    }
}

#[inline(always)]
pub fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[inline(always)]
pub fn align_up_modulo(value: usize, align: usize) -> usize {
    value + (align - (value % align))
}


/// Generate a unique u16 value starting from 1
pub fn gen_u32() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static VALUE: AtomicU32 = AtomicU32::new(1);
    VALUE.fetch_add(1, Relaxed)
}
