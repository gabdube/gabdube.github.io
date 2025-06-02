mod gpu_shared;
pub use gpu_shared::*;

mod message;
use message::*;

pub mod protocol;

use zerocopy::{IntoBytes, Immutable};
use crate::data::{base::BaseSprite, world::World};
use super::GameClient;

/// Temporary storage for sprites when regrouping by texture_id and y position
/// Y component might not be equal to `sprite.position.y` in the case of "floating" objects
#[derive(Copy, Clone)]
pub struct TempSprite {
    pub y: f32,
    pub sprite: BaseSprite,
}

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
    /// Temporary buffer to order the sprites by Y order
    order_sprites: Vec<TempSprite>
}

impl GameOutput {

    pub fn update(client: &mut GameClient) {
        client.output.clear_index();

        if client.data.globals.flags.update_terrain() {
            GameOutput::update_terrain(client);
            client.data.globals.flags.clear_update_terrain();
        }

        if client.data.globals.total_sprites > 0 {
            GameOutput::render_sprites(client);
        }

        if client.data.debug.any() {
            GameOutput::render_debug(client);
        }

        if client.data.globals.flags.update_gui() {
            GameOutput::render_gui(client);
            client.data.globals.flags.clear_update_gui();
        }

        client.output.write_index();
    }

    fn render_sprites(client: &mut GameClient) {
        fn gen_sprites(world: &mut World, buffer: &mut Vec<TempSprite>) {
            for (_, &sprite) in world.iter_all_sprites() {
                buffer.push(TempSprite { y: sprite.position.y, sprite })
            }
        }

        fn gen_sprites_with_animations(world: &mut World, buffer: &mut Vec<TempSprite>) {
            for (_, (sprite, animation)) in world.iter_animated_sprites() {
                animation.current_frame += 1;
                animation.current_frame = animation.current_frame * ((animation.current_frame < animation.max_frame) as u16);
                sprite.texcoord = animation.current_frame();
                buffer.push(TempSprite { y: sprite.position.y, sprite: *sprite })
            }
            
            for (_, &sprite) in world.iter_static_sprites() {
                buffer.push(TempSprite { y: sprite.position.y, sprite })
            }
        }

        fn order_sprites(sprites: &mut Vec<TempSprite>) {
            use std::cmp::Ordering;

            // Sprites with a lower Y value gets rendered first
            // y is always be a valid number, so we don't need to use `total_cmp`
            sprites.sort_unstable_by(|v1, v2| {
                v1.y.partial_cmp(&v2.y).unwrap_or(Ordering::Equal)
            });
        }

        fn gen_commands(output: &mut GameOutput, texture_id: u32) {
            let mut update_sprites = UpdateSpritesParams { offset_bytes: output.data_offset, size_bytes: 0 };
            let mut draw_sprites = DrawSpritesParams { instance_base: 0, instance_count: 0, texture_id };

            for i in 0..output.order_sprites.len() {
                let sprite = output.order_sprites[i].sprite;
                let [width, height] = sprite.texcoord.splat_size();

                // Note: The "position" of a sprite in the display is the top-left corner
                // however the "position" of a sprite in the game is the bottom-center, so we need to move it.
                let gpu_sprite = GpuSpriteData {
                    position: [
                        sprite.position.x - (width * 0.5),   
                        sprite.position.y - height,
                    ],
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
                ty: OutputMessageType::DrawSprites,
                params: OutputMessageParams { draw_sprites } }
            );

            output.messages.push(OutputMessage { 
                ty: OutputMessageType::UpdateSprites,
                params: OutputMessageParams {
                    update_sprites
                }
            })
        }

        // All sprites use the same texture in this tiny demo
        let flags = &mut client.data.globals.flags;
        let texture_id = client.data.assets.atlas.texture.id;

        if flags.update_animations() {
            gen_sprites_with_animations(&mut client.data.world, &mut client.output.order_sprites);
            flags.clear_update_animations();
        } else {
            gen_sprites(&mut client.data.world, &mut client.output.order_sprites);
        }
        
        order_sprites(&mut client.output.order_sprites);
        gen_commands(&mut client.output, texture_id);
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

    fn clear_index(&mut self) {
        self.data_offset = 0;
        self.messages.clear();
        self.order_sprites.clear();
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
            order_sprites: Vec::with_capacity(32),
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

