use crate::data::gui::{Gui, GuiOutputSprite};
use crate::shared::{ExternalId, Scissor};
use super::gpu_data::GpuGuiVertex;
use super::message::{OutputMessage, OutputMessageType, OutputMessageParams, UpdateGuiMessageParams, DrawGuiMessageParams};
use super::output_data_buffer::OutputDataSubBuffer;
use super::GameOutput;

const NO_TEXTURE: ExternalId = ExternalId(u32::MAX);
const INDEX_SIZE: u32 = size_of::<u16>() as u32;
const VERTEX_SIZE: u32 = size_of::<GpuGuiVertex>() as u32;

pub const BASE_GUI_INDEX_CAPACITY: usize = 768;
pub const BASE_GUI_VERTEX_CAPACITY: usize = 1024;

struct DrawCommandState {
    pub draw_count: u32,
    pub indices_base: u32,
    pub indices_offset: u32,
    pub vertices_base: u32,
    pub vertices_offset: u32,
    pub font_texture_id: ExternalId,
    pub image_texture_id: ExternalId,
    pub scissor: Scissor,
}

pub(super) struct GuiMeshRasterizer<'a> {
    pub(super) gui: &'a mut Gui,
    pub(super) output: &'a mut GameOutput,
}

impl<'a> GuiMeshRasterizer<'a> {

    pub fn generate_meshes(&mut self) -> bool {
        let extra = &self.output.extra;
        let index_capacity = extra.last_gui_index_capacity;
        let vertex_capacity = extra.last_gui_vertex_capacity;

        let (mut indices, mut vertices) = self.output.data.reserve2::<u16, GpuGuiVertex>(
            index_capacity,
            vertex_capacity
        );

        // Tell the engine to clear the last gui state
        Self::clear_gui_message(&mut self.output.messages);

        // Generate the gui mesh
        // Mesh size might exceed the staging buffer capacity, if so the capacity will be reallocated and re-rendered next frame
        let needs_rerender = self.generate_meshes_inner(index_capacity, &mut indices, vertex_capacity, &mut vertices);

        // Release data reserved by reserve2
        self.output.data.release(indices);
        self.output.data.release(vertices);

        needs_rerender
    }

    fn generate_meshes_inner(
        &mut self,
        indices_capacity: usize,
        indices_buffer: &mut OutputDataSubBuffer<u16>,
        vertices_capacity: usize,
        vertices_buffer: &mut OutputDataSubBuffer<GpuGuiVertex>
    ) -> bool 
    {
        fn clear_command_state(indices_base: u32, vertices_base: u32, scissor: Scissor) -> DrawCommandState {
            DrawCommandState { 
                draw_count: 0,
                indices_base,
                indices_offset: 0,
                vertices_base,
                vertices_offset: 0,
                font_texture_id: NO_TEXTURE,
                image_texture_id: NO_TEXTURE,
                scissor,
            }
        }

        /// Each draw message is mapped into a draw call on the engine side
        /// Changing the resource bindings or the scissor state means we need to generate a new draw message 
        fn must_generate_draw_message(sprite: &GuiOutputSprite, state: &DrawCommandState) -> bool {
            let new_font = sprite.font_texture_id != NO_TEXTURE && state.font_texture_id != NO_TEXTURE && sprite.font_texture_id != state.font_texture_id;
            let new_image = sprite.image_texture_id != NO_TEXTURE && state.image_texture_id != NO_TEXTURE && sprite.image_texture_id != state.image_texture_id;
            let new_scissor = sprite.scissor != state.scissor;
            new_font || new_image || new_scissor
        }

        fn generate_vertices(indices: &mut [u16], vertices: &mut [GpuGuiVertex], state: &mut DrawCommandState, sprite: &GuiOutputSprite) {
            let i = (state.indices_base + state.indices_offset) as usize;
            let v = state.vertices_offset as u16;
            indices[i..i+6].copy_from_slice(&[v+0, v+1, v+2, v+0, v+2, v+3]);

            let v = (state.vertices_base + state.vertices_offset) as usize;
            let [left, top, right, bottom] = sprite.positions.splat();
            let [tleft, ttop, tright, tbottom] = sprite.texcoord.splat();
            let color = sprite.color.splat();
            let data = sprite.flags.0;
            vertices[v+0] = GpuGuiVertex { position: [left, top],     texcoord: [tleft, ttop],     color, data };
            vertices[v+1] = GpuGuiVertex { position: [left, bottom],  texcoord: [tleft, tbottom],  color, data };
            vertices[v+2] = GpuGuiVertex { position: [right, bottom], texcoord: [tright, tbottom], color, data };
            vertices[v+3] = GpuGuiVertex { position: [right, top],    texcoord: [tright, ttop],    color, data };

            state.draw_count += 6;
            state.indices_offset += 6;
            state.vertices_offset += 4;
            state.scissor = sprite.scissor;
            if sprite.font_texture_id  != NO_TEXTURE { state.font_texture_id = sprite.font_texture_id; }
            if sprite.image_texture_id != NO_TEXTURE { state.image_texture_id = sprite.image_texture_id; }
        }

        let messages = &mut self.output.messages;
        let indices = &mut indices_buffer.data;
        let vertices = &mut vertices_buffer.data;

        let base_scissor = self.gui.base_scissor();
        let mut state = clear_command_state(0, 0, base_scissor);

        let mut total_indices_count = 0;
        let mut total_vertices_count = 0;

        let needs_rerender = self.gui.generate_sprites(|sprite| {
            total_indices_count += 6;
            total_vertices_count += 4;

            if total_indices_count < indices_capacity && total_vertices_count < vertices_capacity {
                if must_generate_draw_message(&sprite, &state) {
                    Self::draw_gui_message(messages, &state);
                    state = clear_command_state(
                        state.indices_base + state.indices_offset,
                        state.vertices_base + state.vertices_offset,
                        base_scissor
                    );
                }

                generate_vertices(indices, vertices, &mut state, &sprite);
            }
        });

        if total_indices_count > 0 {
            Self::draw_gui_message(messages, &state);

            Self::update_gui_message(
                &mut self.output.messages,
                indices_buffer.data_bytes_offset as u32,
                total_indices_count as u32,
                vertices_buffer.data_bytes_offset as u32,
                total_vertices_count as u32
            );
        }

        // If the number of generated vertices exceed the staging buffer size, we record the size
        // of the output and will resize the staging buffer on the next frame and then rerender the GUI
        if total_indices_count > indices_capacity || total_vertices_count > vertices_capacity {
            use crate::shared::align_up_modulo;
            let extra = &mut self.output.extra;
            extra.last_gui_index_capacity = align_up_modulo(total_indices_count, BASE_GUI_INDEX_CAPACITY);
            extra.last_gui_vertex_capacity = align_up_modulo(total_vertices_count, BASE_GUI_VERTEX_CAPACITY);
            true
        } else {
            needs_rerender
        }
    }

    fn clear_gui_message(messages: &mut Vec<OutputMessage>) {
        messages.push(OutputMessage {
            ty: OutputMessageType::ClearGui,
            params: OutputMessageParams { none: () }
        });
    }

    /// The "update_gui" message sends the whole GUI mesh to the engine
    fn update_gui_message(
        messages: &mut Vec<OutputMessage>,
        index_bytes_offset: u32,
        index_size: u32,
        vertex_bytes_offset: u32,
        vertex_size: u32
    ) {
        let update_gui = UpdateGuiMessageParams {
            index_bytes_offset,
            index_bytes_size: index_size * INDEX_SIZE,
            vertex_bytes_offset,
            vertex_bytes_size: vertex_size * VERTEX_SIZE,
        };

        messages.push(OutputMessage {
            ty: OutputMessageType::UpdateGui,
            params: OutputMessageParams { update_gui }
        });
    }

    /// The "draw_gui" tells the engine how to render the current GUI state
    /// Each draw_gui message is mapped to a unique draw call
    fn draw_gui_message(
        messages: &mut Vec<OutputMessage>,
        state: &DrawCommandState,
    ) {
        if state.draw_count == 0 {
            return;
        }

        let draw_gui = DrawGuiMessageParams {
            draw_count: state.draw_count,
            index_bytes_offset: state.indices_base * INDEX_SIZE,
            vertex_bytes_offset: state.vertices_base * VERTEX_SIZE,
            image_texture: state.image_texture_id.0,
            font_texture: state.font_texture_id.0,
            scissor: state.scissor.splat()
        };

        messages.push(OutputMessage {
            ty: OutputMessageType::DrawGui,
            params: OutputMessageParams { draw_gui }
        });
    }

}
