use zerocopy_derive::{Immutable, IntoBytes};

/// Information on how to render a terrain cell on the GPU
#[repr(C)]
#[derive(Default, Copy, Clone, Immutable, IntoBytes)]
pub struct GpuTerrainSpriteData {
    pub position: [f32; 2],
    pub uv: [f32; 2]
}