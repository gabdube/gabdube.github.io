mod gpu_shared;
pub use gpu_shared::*;

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

    pub fn update(client: &mut GameClient) {
        let mut flags = client.data.globals.flags;

        client.output.clear_index();

        if flags.update_view_offset() {
            GameOutput::update_view_offset(client);
            flags.clear_update_view_offset();
        }

        if flags.update_terrain() {
            GameOutput::update_terrain(client);
            flags.clear_update_terrain();
        }

        if client.data.globals.total_sprites > 0 {
            GameOutput::render_sprites(client);
            flags.clear_update_animations();
        }

        if let Some(sprite) = client.data.world.has_insert_sprite() {
            GameOutput::render_insert_sprite(client, sprite);
        }

        if client.data.debug.any() {
            GameOutput::render_debug(client);
        }

        if flags.update_gui() {
            GameOutput::render_gui(client);
            flags.clear_update_gui();
        }

        client.data.globals.flags = flags;
        client.output.write_index();
    }

    fn render_sprites(client: &mut GameClient) {
        // All sprites use the same texture in this tiny demo
        let flags = &mut client.data.globals.flags;
        let texture_id = client.data.assets.atlas.texture.id;
        let output = &mut client.output;

        client.data.world.order_sprites(flags.update_animations());
        
        let mut update_sprites = UpdateSpritesParams { offset_bytes: output.data_offset, size_bytes: 0 };
        let mut draw_sprites = DrawSpritesParams { instance_base: 0, instance_count: 0, texture_id };

        for sprite in client.data.world.ordered_sprites() {
            let [width, height] = sprite.texcoord.splat_size();

            let gpu_sprite = GpuSpriteData {
                position: sprite.position.splat(),
                size: [width, height],
                texcoord_offset: [sprite.texcoord.left, sprite.texcoord.top],
                texcoord_size: [width, height],
                data: sprite.flags.value()
            };

            output.push_data(&gpu_sprite);

            draw_sprites.instance_count += 1;
            update_sprites.size_bytes += size_of::<GpuSpriteData>();
        }

        output.messages.push(OutputMessage { 
            ty: OutputMessageType::UpdateSprites,
            params: OutputMessageParams {
                update_sprites
            }
        });

        output.messages.push(OutputMessage { 
            ty: OutputMessageType::DrawSprites,
            params: OutputMessageParams { draw_sprites } }
        );
    }

    fn render_insert_sprite(client: &mut GameClient, insert_sprite: crate::data::world::InsertSprite) {
        let vertex_offset_bytes = client.output.data_offset;
        let vertex_size_bytes = size_of::<InsertSpriteVertex>() * 6;
        
        let [tx1, ty1, tx2, ty2] = insert_sprite.sprite.splat();
        let [width, height] = insert_sprite.sprite.splat_size();
        let [x1, y1] = insert_sprite.position.splat();
        let [x2, y2] = [x1 + width, y1 + height];

        client.output.push_data(&[
            InsertSpriteVertex { position: [x1, y1], texcoord: [tx1, ty1] },
            InsertSpriteVertex { position: [x1, y2], texcoord: [tx1, ty2] },
            InsertSpriteVertex { position: [x2, y2], texcoord: [tx2, ty2] },

            InsertSpriteVertex { position: [x2, y1], texcoord: [tx2, ty1] },
            InsertSpriteVertex { position: [x1, y1], texcoord: [tx1, ty1] },
            InsertSpriteVertex { position: [x2, y2], texcoord: [tx2, ty2] },
        ]);
        
        let params = DrawInsertSpriteParams {
            vertex_offset_bytes,
            vertex_size_bytes
        };
        client.output.messages.push(OutputMessage { 
            ty: OutputMessageType::DrawInsertSprite,
            params: OutputMessageParams { draw_insert_sprite: params },
        });
    }

    fn update_terrain(client: &mut GameClient) {
        const TERRAIN_SPRITE_SIZE: f32 = 64.0;

        let data = &client.data;
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
        let mut sprite = GpuTerrainSpriteData::default();
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

    fn render_debug(client: &mut GameClient) {
        let output = &mut client.output;

        // Preallocating vertex & index space
        let [index_count, index_size, vertex_size] = client.data.debug.buffers_sizes();
        let total_size = index_size + vertex_size;
        if output.data[output.data_offset..].len() < total_size {
            output.realloc_data(total_size);
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
        client.data.debug.generate_mesh(index_slice, vertex_slice);

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

    #[cfg(feature="gui")]
    fn update_gui_textures(&mut self, delta: &egui::TexturesDelta) {
        for (id, delta) in &delta.set {
            // Upload data
            let pixels_offset;
            let pixels_size;
            let [x, y] = delta.pos.unwrap_or([0, 0]);
            let [width, height] = delta.image.size();
            match &delta.image {
                egui::ImageData::Color(_image) => { panic!("Unsupported"); },
                egui::ImageData::Font(image) => {
                    let data: Vec<u8> = image.srgba_pixels(None).flat_map(|a| a.to_array() ).collect();
                    pixels_size = data.len();
                    pixels_offset = self.push_bytes(&data);
                }
            }

            // Message
            let gui_texture_update = GuiTextureUpdateParams {
                pixels_offset,
                pixels_size,
                x: x as u32,
                y: y as u32,
                width: width as u32,
                height: height as u32,
                id: match id {
                    egui::TextureId::Managed(x) => *x as u32,
                    egui::TextureId::User(x) => *x as u32,
                }
            };

            self.messages.push(OutputMessage { 
                ty: OutputMessageType::GuiTextureUpdate,
                params: OutputMessageParams { gui_texture_update } }
            );
        }
    }

    #[cfg(feature="gui")]
    fn update_gui_mesh(&mut self, mesh: &Vec<egui::ClippedPrimitive>) {
        use egui::epaint::{Primitive, Vertex, Rect};

        fn update_mesh(clip: &Rect, mesh: &egui::Mesh, output: &mut GameOutput) {
            let index_offset_bytes = output.push_bytes(&mesh.indices);
            let vertex_offset_bytes = output.push_bytes(&mesh.vertices);
            let index_size_bytes = mesh.indices.len() * size_of::<u32>();
            let vertex_size_bytes = mesh.vertices.len() * size_of::<Vertex>();
            let gui_mesh_update = GuiMeshUpdateParams {
                index_offset_bytes,
                index_size_bytes,
                vertex_offset_bytes,
                vertex_size_bytes,
                count: mesh.indices.len() as u32,
                clip: [clip.min.x, clip.min.y, clip.max.x, clip.max.y],
                texture_id: match mesh.texture_id {
                    egui::TextureId::Managed(x) => x as u32,
                    egui::TextureId::User(x) => x as u32,
                }
            };

            output.messages.push(OutputMessage { 
                ty: OutputMessageType::GuiMeshUpdate,
                params: OutputMessageParams { gui_mesh_update } }
            );
        }

        for clipped_primitive in mesh.iter() {
            match &clipped_primitive.primitive {
                Primitive::Callback(_) => { panic!("Unsupported") },
                Primitive::Mesh(mesh) => {
                    update_mesh(&clipped_primitive.clip_rect, mesh, self);
                }
            }
        }
    }
    
    #[cfg(feature="gui")]
    fn render_gui(client: &mut GameClient) {
        let output = &mut client.output;

        output.messages.push(OutputMessage { ty: OutputMessageType::ResetGui, params: OutputMessageParams { none: () } });

        let delta = client.data.gui.texture_delta();
        output.update_gui_textures(&delta);

        let mesh = client.data.gui.tesselate();
        output.update_gui_mesh(&mesh);
    }

    #[cfg(not(feature="gui"))]
    fn render_gui(_client: &mut GameClient) {}

    fn update_view_offset(client: &mut GameClient) {
        client.output.messages.push(OutputMessage { 
            ty: OutputMessageType::UpdateViewOffset,
            params: OutputMessageParams { update_view_offset: client.data.globals.view_offset },
        });
    }

    fn clear_index(&mut self) {
        self.data_offset = 0;
        self.messages.clear();
    }

    fn write_index(&mut self) {
        self.output_index.messages_count = self.messages.len();
        self.output_index.messages_ptr = self.messages.as_ptr();
        self.output_index.data_ptr = self.data.as_ptr();
    }

    fn push_data<T: IntoBytes+Immutable>(&mut self, data: &T) {
        let size = size_of::<T>();
        if self.data[self.data_offset..].len() < size {
            self.realloc_data(size)
        }

        if let Err(_) = data.write_to_prefix(&mut self.data[self.data_offset..]) {
            unsafe { std::hint::unreachable_unchecked() } // Safety. Capacity check above ensure this this never be reached
        }

        self.data_offset += size;
    }

    fn push_bytes<T: Copy>(&mut self, data: &[T]) -> usize {
        let (_, bytes, _) = unsafe { data.align_to::<u8>() };
        let data_offset = crate::shared::align_up(self.data_offset, align_of::<T>());

        let size = bytes.len();
        if self.data[data_offset..].len() < size {
            self.realloc_data(size)
        }

        unsafe {
            ::std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.data[data_offset..].as_mut_ptr(), size);
        }

        self.data_offset = data_offset + size;
        data_offset
    }

    #[inline(never)]
    #[cold]
    fn realloc_data(&mut self, min_size: usize) {
        self.data.reserve_exact(crate::shared::align_up(min_size, 0x8000));
        unsafe { self.data.set_len(self.data.capacity()); }
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

