mod gpu_shared;
use gpu_shared::*;

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

        if client.data.globals.total_sprites > 0 {
            GameOutput::render_sprites(client);
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

    fn clear_index(&mut self) {
        self.data_offset = 0;
        self.data.clear();

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

    #[inline(never)]
    #[cold]
    fn realloc_data(&mut self, min_size: usize) {
        self.data.reserve_exact(crate::shared::align_up(min_size, 0x1000));
        unsafe { self.data.set_len(self.data.capacity()); }
    }

}

impl Default for GameOutput {

    fn default() -> Self {
        let output_index: Box<OutputIndex> = Box::default();
        GameOutput {
            output_index: Box::leak(output_index),
            messages: Vec::with_capacity(16),
            data: vec![0; 0x1000],
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

