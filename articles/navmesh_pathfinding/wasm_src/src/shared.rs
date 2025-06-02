use zerocopy_derive::{Immutable, IntoBytes, FromBytes};

#[derive(Default, Debug, Copy, Clone, PartialEq, FromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct PositionF32 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, FromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct SizeF32 {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Default, FromBytes, IntoBytes, Immutable)]
pub struct AABB {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32
}

impl AABB {
    pub fn splat_size(&self) -> [f32; 2] {
        [self.right - self.left, self.bottom - self.top]
    }

    pub fn size(&self) -> SizeF32 {
        SizeF32 { width: self.right - self.left, height: self.bottom - self.top }
    }
}

//
// Helpers method
//

pub const fn pos(x: f32, y: f32) -> PositionF32 {
    PositionF32 { x, y }
}

pub const fn size(width: f32, height: f32) -> SizeF32 {
    SizeF32 { width, height }
}

pub const fn aabb(position: PositionF32, size: SizeF32) -> AABB {
    AABB {
        left: position.x,
        top: position.y,
        right: position.x + size.width,
        bottom: position.y + size.height
    }
}

/// Split a csv string into up to `MAX_ARGS` parameters. Calls `callback` for each line splitted.
pub fn split_csv<const MAX_ARGS: usize, CB: FnMut(&[&str])>(csv: &str, mut callback: CB) {
    let mut start = 0;
    let mut end = 0;
    let mut chars_iter = csv.chars();
    let mut args: [&str; MAX_ARGS] = [""; MAX_ARGS];
    while let Some(c) = chars_iter.next() {
        end += 1;
        if c == '\n' {
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
