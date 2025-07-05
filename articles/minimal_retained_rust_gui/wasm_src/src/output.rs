mod gpu_shared;

mod message;
use message::*;

pub mod protocol;

use zerocopy::{IntoBytes, Immutable};
use super::GameClient;

/// The index of all the pointers and array size to share with the engine
/// Must be `repr(C)` because it will be directly read from memory by the engine
#[repr(C)]
pub struct OutputIndex {
    pub messages_count: usize,
    pub messages_size: usize,
    pub messages_ptr: *const OutputMessage,
    pub data_ptr: *const u8,
}


/// Holds the data buffer shared between the game client and the engine 
pub struct GameOutput {
    /// This is a leaked box because we return the pointer to the client in `output` and `Box::as_ptr` is a nightly-only experimental API
    pub output_index: &'static mut OutputIndex,
    /// High level rendering commmand shared with the engine
    messages: Vec<OutputMessage>,
    /// Generic data storage shared with the engine
    data: Vec<u8>,
    data_offset: usize,
}

impl GameOutput {

    fn clear_index(&mut self) {
        self.data_offset = 0;
        self.messages.clear();
    }

    fn write_index(&mut self) {
        self.output_index.messages_count = self.messages.len();
        self.output_index.messages_ptr = self.messages.as_ptr();
        self.output_index.data_ptr = self.data.as_ptr();
    }

    pub fn update(client: &mut GameClient) {
        let flags = client.world_data.data.common.render_flags;
        client.output.clear_index();

        if flags.update_view_offset() {
            GameOutput::update_view_offset(client);
        }

        if flags.update_zoom() {
            GameOutput::update_view_size(client);
        }

        if flags.update_terrain() {
            GameOutput::update_terrain(client);
        }

        client.world_data.data.common.render_flags.clear();
        client.output.write_index();
    }

    fn update_view_offset(client: &mut GameClient) {
        client.output.messages.push(OutputMessage { 
            ty: OutputMessageType::UpdateViewOffset,
            params: OutputMessageParams { update_view_offset: client.world_data.data.common.view_offset },
        });
    }

    fn update_view_size(client: &mut GameClient) {
        let common = &client.world_data.data.common;
        let width = common.view_size.width * (1.0 / common.zoom);
        let height = common.view_size.height * (1.0 / common.zoom);

        client.output.messages.push(OutputMessage { 
            ty: OutputMessageType::UpdateViewSize,
            params: OutputMessageParams { update_view_size: crate::shared::size(width, height) },
        });
    }

    fn update_terrain(client: &mut GameClient) {
        use crate::data::terrain::TERRAIN_SPRITE_SIZE;

        let data = &client.world_data.data;
        let output = &mut client.output;

        // Message
        let cell_count = data.terrain.cell_count();
        let update_terrain = UpdateTerrainParams { 
            offset_bytes: output.data_offset,
            size_bytes: cell_count * size_of::<gpu_shared::GpuTerrainSpriteData>(),
            cell_count,
        };

        output.messages.push(OutputMessage { 
            ty: OutputMessageType::UpdateTerrain,
            params: OutputMessageParams { update_terrain } }
        );

        // Data
        let mut x = 0.0;
        let mut y = 0.0;
        let mut sprite = gpu_shared::GpuTerrainSpriteData::default();
        for _ in 0..data.terrain.height() {
            for _ in 0..data.terrain.width() {
                sprite.position = [x, y];
                sprite.uv = [0.0, 0.0];
                output.push_data(&sprite);
                x += TERRAIN_SPRITE_SIZE;
            }

            x = 0.0;
            y += TERRAIN_SPRITE_SIZE;
        }
    }

    fn push_data<T: IntoBytes+Immutable>(&mut self, data: &T) {
        let size = size_of::<T>();
        if self.data[self.data_offset..].len() < size {
            Self::realloc_data(&mut self.data, size);
        }

        if let Err(_) = data.write_to_prefix(&mut self.data[self.data_offset..]) {
            unsafe { std::hint::unreachable_unchecked() } // Safety. Capacity check above ensure this this never be reached
        }

        self.data_offset += size;
    }

    #[inline(never)]
    #[cold]
    fn realloc_data(buffer: &mut Vec<u8>, min_size: usize) {
        buffer.reserve_exact(crate::shared::align_up(min_size, 0x8000));
        unsafe { buffer.set_len(buffer.capacity()); }
    }
}

impl Default for GameOutput {

    fn default() -> Self {
        let output_index: Box<OutputIndex> = Box::default();
        GameOutput {
            output_index: Box::leak(output_index),
            messages: Vec::with_capacity(16),
            data: vec![0; 0xF0000],
            data_offset: 0,
        }
    }

}

impl Default for OutputIndex {
    fn default() -> Self {
        OutputIndex {
            messages_count: 0,
            messages_size: size_of::<OutputMessage>(),
            messages_ptr: ::std::ptr::null(),
            data_ptr: ::std::ptr::null(),
        }
    }
}

