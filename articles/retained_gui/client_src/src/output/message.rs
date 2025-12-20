#[repr(C)]
#[derive(Copy, Clone)]
pub struct UpdateGuiMessageParams {
    pub index_bytes_offset: u32,
    pub index_bytes_size: u32,
    pub vertex_bytes_offset: u32,
    pub vertex_bytes_size: u32
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawGuiMessageParams {
    pub draw_count: u32,
    pub index_bytes_offset: u32,
    pub vertex_bytes_offset: u32,
    pub image_texture: u32,
    pub font_texture: u32,
    pub scissor: [u16; 4],
}

// Note: This is a union!
#[repr(C)]
#[derive(Copy, Clone)]
pub union OutputMessageParams {
    pub none: (),
    pub update_gui: UpdateGuiMessageParams,
    pub draw_gui: DrawGuiMessageParams,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum OutputMessageType {
    ClearGui=1,
    UpdateGui,
    DrawGui,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OutputMessage {
    pub ty: OutputMessageType,
    pub params: OutputMessageParams
}
