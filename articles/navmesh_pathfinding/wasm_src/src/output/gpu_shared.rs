//! Strutures of data shared between the game and the GPU
use zerocopy_derive::*;

/// Information on how to render a sprite on the GPU
/// Memory layout must match `in_instance_position`, `in_instance_texcoord`, `in_instance_data` in `sprites.vert.glsl`
#[repr(C)]
#[derive(Copy, Clone, Immutable, IntoBytes)]
pub struct GpuSpriteData {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub texcoord_offset: [f32; 2],
    pub texcoord_size: [f32; 2],
    pub data: i32,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Immutable, IntoBytes)]
pub struct GpuTerrainSpriteData {
    pub position: [f32; 2],
    pub uv: [f32; 2]
}

#[repr(C)]
#[derive(Default, Copy, Clone, Immutable, IntoBytes)]
pub struct GpuDebugVertex {
    pub position: [f32; 2],
    pub color: [u8; 4]
}

#[repr(C)]
#[derive(Default, Copy, Clone, Immutable, IntoBytes)]
pub struct InsertSpriteVertex {
    pub position: [f32; 2],
    pub texcoord: [f32; 2]
}
