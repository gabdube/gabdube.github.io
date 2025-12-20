use zerocopy_derive::*;

#[repr(C)]
#[derive(Copy, Clone, Immutable, IntoBytes)]
pub struct GpuGuiVertex {
    pub position: [f32; 2],
    pub texcoord: [f32; 2],
    pub color: [u8; 4],
    pub data: u32,
}
