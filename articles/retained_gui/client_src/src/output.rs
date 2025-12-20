mod gpu_data;

mod message;
use message::*;

mod output_data_buffer;
use output_data_buffer::OutputDataBuffer;

mod gui_rasterizer;
use gui_rasterizer::GuiMeshRasterizer;

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

pub struct OutputExtra {
    pub last_gui_index_capacity: usize,
    pub last_gui_vertex_capacity: usize,
}

/// Holds the data buffer shared between the game client and the engine 
pub struct GameOutput {
    /// This is a leaked box because we return the pointer to the client in `output` and `Box::as_ptr` is a nightly-only experimental API
    pub output_index: &'static mut OutputIndex,
    /// High level rendering commmand shared with the engine
    messages: Vec<OutputMessage>,
    /// Data shared with the engine
    data: OutputDataBuffer,
    /// Extra variable used by the renderer
    extra: OutputExtra,
}

impl GameOutput {
    
    fn clear_index(&mut self) {
        self.data.clear();
        self.messages.clear();
        *self.output_index = OutputIndex::default();
    }

    fn write_index(&mut self) {
        self.output_index.messages_size = size_of::<OutputMessage>();
        self.output_index.messages_count = self.messages.len();
        self.output_index.messages_ptr = self.messages.as_ptr();
        self.output_index.data_ptr = self.data.as_ptr();
    }

    pub fn update(client: &mut GameClient) {
        client.output.clear_index();
        Self::generate_index(client);
        client.output.write_index();
    }

    fn generate_index(client: &mut GameClient) {
        let flags = client.data.take_render_flags();

        if flags.update_gui() {
            GameOutput::render_gui(client);
        }
    }

    fn render_gui(client: &mut GameClient) {
        let mut raster = GuiMeshRasterizer {
            gui: &mut client.data.gui,
            output: &mut client.output,
        };

        let needs_rerender = raster.generate_meshes();
        if needs_rerender {
            client.data.update_gui();
        }
    }
    
}

impl Default for GameOutput {
    fn default() -> Self {
        let output_index: Box<OutputIndex> = Box::default();
        let extra = OutputExtra { 
            last_gui_index_capacity: gui_rasterizer::BASE_GUI_INDEX_CAPACITY,
            last_gui_vertex_capacity: gui_rasterizer::BASE_GUI_VERTEX_CAPACITY
        };

        GameOutput {
            output_index: Box::leak(output_index),
            messages: Vec::with_capacity(16),
            data: OutputDataBuffer::with_capacity(0x10000),
            extra,
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
