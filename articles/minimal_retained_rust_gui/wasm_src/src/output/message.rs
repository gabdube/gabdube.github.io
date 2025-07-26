use crate::shared::{PositionF32, SizeF32};

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
    pub background_cell_count: usize,
    pub foreground_cell_count: usize,
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
pub struct DrawInsertSpriteParams {
    pub vertex_offset_bytes: usize,
    pub vertex_size_bytes: usize,
}

// Note: This is a union!
#[repr(C)]
#[derive(Copy, Clone)]
pub union OutputMessageParams {
    pub none: (),
    pub update_sprites: UpdateSpritesParams,
    pub draw_sprites: DrawSpritesParams,
    pub update_highlight_sprites: UpdateSpritesParams,
    pub highlight_sprites: DrawSpritesParams,
    pub update_terrain: UpdateTerrainParams,
    pub draw_debug: DrawDebugParams,
    pub update_view_offset: PositionF32,
    pub update_view_size: SizeF32,
    pub draw_insert_sprite: DrawInsertSpriteParams,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum OutputMessageType {
    UpdateSprites,
    DrawSprites,
    UpdateHighlightSprites,
    HighlightSprites,
    UpdateTerrain,
    DrawDebug,
    ResetGui,
    UpdateViewOffset,
    UpdateViewSize,
    DrawInsertSprite,
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
