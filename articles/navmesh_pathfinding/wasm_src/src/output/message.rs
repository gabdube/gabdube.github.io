/// Engine must read data in the client data buffer and copy it in the engine sprite instance buffer
#[repr(C)]
#[derive(Copy, Clone)]
pub struct UpdateSpritesParams {
    /// The offset in the client data buffer
    pub offset_bytes: usize,
    /// The size to copy
    pub size_bytes: usize,
}

/// Engine must draw `instance_count` sprites using the "Sprites" shader, starting at `instance_base` and using `texture_id`
/// Data comes from the `UpdateSprites` command
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawSpritesParams {
    pub instance_base: u32,
    pub instance_count: u32,
    pub texture_id: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UpdateTerrainParams {
    pub offset_bytes: usize,
    pub size_bytes: usize,
    pub cell_count: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawDebugParams {
    pub index_offset_bytes: usize,
    pub index_size_bytes: usize,
    pub vertex_offset_bytes: usize,
    pub vertex_size_bytes: usize,
    pub count: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GuiTextureUpdateParams {
    pub pixels_offset: usize,
    pub pixels_size: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub id: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GuiMeshUpdateParams {
    pub index_offset_bytes: usize,
    pub index_size_bytes: usize,
    pub vertex_offset_bytes: usize,
    pub vertex_size_bytes: usize,
    pub clip: [f32; 4],
    pub count: u32,
    pub texture_id: u32,
}

// Note: This is a union!
#[repr(C)]
#[derive(Copy, Clone)]
pub union OutputMessageParams {
    pub none: (),
    pub update_sprites: UpdateSpritesParams,
    pub draw_sprites: DrawSpritesParams,
    pub update_terrain: UpdateTerrainParams,
    pub draw_debug: DrawDebugParams,
    pub gui_texture_update: GuiTextureUpdateParams,
    pub gui_mesh_update: GuiMeshUpdateParams,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum OutputMessageType {
    UpdateSprites,
    DrawSprites,
    UpdateTerrain,
    DrawDebug,
    GuiTextureUpdate,
    GuiMeshUpdate,
    ResetGui,
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
