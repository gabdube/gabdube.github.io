mod gpu_shared;

mod message;
use message::*;

mod debug_rasterizer;
use debug_rasterizer::DebugMeshRasterizer;

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
    /// Temporary storage for highlighted sprites
    highlighted: Vec<gpu_shared::GpuHighlightedSprite>,
}

impl GameOutput {

    fn clear_index(&mut self) {
        self.data_offset = 0;
        self.messages.clear();
        self.highlighted.clear()
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

        if client.world_data.data.common.total_sprites > 0 {
            GameOutput::render_sprites(client);
        }

        if client.world_data.data.debug.any() {
            GameOutput::render_debug(client);
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

    fn render_sprites(client: &mut GameClient) {
        use gpu_shared::{GpuSpriteData, GpuHighlightedSprite};

        // All sprites use the same texture in this tiny demo
        let data = &mut client.world_data.data;
        let flags = &mut data.common.render_flags;
        let texture_id = data.assets.atlas.texture.id;
        let output = &mut client.output;

        let instance_count = client.world_data.world.order_sprites(flags.update_animations());
        
        let update_sprites = UpdateSpritesParams { 
            offset_bytes: output.data_offset,
            size_bytes: instance_count * size_of::<GpuSpriteData>(),
        };
        output.messages.push(OutputMessage { 
            ty: OutputMessageType::UpdateSprites,
            params: OutputMessageParams { update_sprites }
        });

        let draw_sprites = DrawSpritesParams { 
            instance_base: 0,
            instance_count: instance_count as u32,
            texture_id
        };
        output.messages.push(OutputMessage { 
            ty: OutputMessageType::DrawSprites,
            params: OutputMessageParams { draw_sprites } }
        );

        for sprite in client.world_data.world.ordered_sprites() {
            let [width, height] = sprite.texcoord.splat_size();

            let mut gpu_sprite = GpuSpriteData {
                position: sprite.position.splat(),
                size: [width, height],
                texcoord_offset: [sprite.texcoord.left, sprite.texcoord.top],
                texcoord_size: [width, height],
            };

            if sprite.flags.flipped() {
                gpu_sprite.texcoord_offset[0] += width;
                gpu_sprite.texcoord_size[0] = -gpu_sprite.texcoord_size[0];
            }

            output.push_data(&gpu_sprite);

            if sprite.flags.highlighted() {
                let [r, g, b] = sprite.highlight_color;
                let highlighted = GpuHighlightedSprite {
                    position: gpu_sprite.position,
                    size: gpu_sprite.size,
                    texcoord_offset: gpu_sprite.texcoord_offset,
                    texcoord_size: gpu_sprite.texcoord_size,
                    highlight: [r, g, b, 255],
                };
                output.highlighted.push(highlighted);
            }
        }
      
        // Highlighted sprites
        if output.highlighted.is_empty() {
            return;
        }

        let highlighted_count = output.highlighted.len();
        let update_highlight_sprites = UpdateSpritesParams { 
            offset_bytes: output.data_offset,
            size_bytes: highlighted_count * size_of::<GpuHighlightedSprite>(),
        };
        output.messages.push(OutputMessage { 
            ty: OutputMessageType::UpdateHighlightSprites,
            params: OutputMessageParams { update_highlight_sprites }
        });

        let highlight_sprites = DrawSpritesParams { 
            instance_base: 0,
            instance_count: highlighted_count as u32,
            texture_id
        };
        output.messages.push(OutputMessage { 
            ty: OutputMessageType::HighlightSprites,
            params: OutputMessageParams { highlight_sprites } }
        );

        Self::push_bytes_inner(&output.highlighted, &mut output.data, &mut output.data_offset);
    }

    fn render_debug(client: &mut GameClient) {
        let output = &mut client.output;
        let debug = &client.world_data.data.debug;

        // Preallocating vertex & index space
        let [index_count, index_size, vertex_size] = DebugMeshRasterizer::buffers_sizes(debug);
        let total_size = index_size + vertex_size;
        if output.data[output.data_offset..].len() < total_size {
            Self::realloc_data(&mut output.data, total_size);
        }

        output.data_offset = crate::shared::align_up(output.data_offset, 4);
        let index_offset_base = output.data_offset;
        let vertex_offset_base = index_offset_base + index_size;
        output.data_offset += total_size;

        // Generating vertex & indices
        let (data, _) = output.data.split_at_mut(output.data_offset);
        let (data, vertex_slice) = data.split_at_mut(vertex_offset_base);
        let (_, index_slice) = data.split_at_mut(index_offset_base);
        assert!(index_slice.len() == index_size && vertex_slice.len() == vertex_size);
        DebugMeshRasterizer::generate_mesh(debug, index_slice, vertex_slice);

        // Message generation
        let draw_debug = DrawDebugParams {
            index_offset_bytes: index_offset_base,
            index_size_bytes: index_size,
            vertex_offset_bytes: vertex_offset_base,
            vertex_size_bytes: vertex_size,
            count: index_count
        };

        client.output.messages.push(OutputMessage { 
            ty: OutputMessageType::DrawDebug,
            params: OutputMessageParams { draw_debug } }
        );
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

    fn push_bytes_inner<T: Copy>(src: &[T], dst: &mut Vec<u8>, offset: &mut usize) -> usize {
        let data_offset = crate::shared::align_up(*offset, align_of::<T>());
        let (_, bytes, _) = unsafe { src.align_to::<u8>() };

        let size = bytes.len();
        if dst[data_offset..].len() < size {
            Self::realloc_data(dst, size);
        }

        unsafe {
            ::std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst[data_offset..].as_mut_ptr(), size);
        }

        *offset = data_offset + size;

        data_offset
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
            highlighted: Vec::with_capacity(16)
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

