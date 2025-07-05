use crate::shared::{PositionF32, SizeF32};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UpdateTerrainParams {
    pub offset_bytes: usize,
    pub size_bytes: usize,
    pub cell_count: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union OutputMessageParams {
    pub none: (),
    pub update_terrain: UpdateTerrainParams,
    pub update_view_offset: PositionF32,
    pub update_view_size: SizeF32,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum OutputMessageType {
    UpdateTerrain,
    UpdateViewOffset,
    UpdateViewSize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OutputMessage {
    pub ty: OutputMessageType,
    pub params: OutputMessageParams
}


//
// Other impl
//

impl Into<u32> for OutputMessageType {
    fn into(self) -> u32 {
        self as u32
    }
}

