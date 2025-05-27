use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::output::GpuDebugVertex;
use crate::shared::AABB;

#[derive(Copy, Clone, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
pub enum DebugElement {
    Rect { base: AABB, line_thickness: f32, color: [u8; 4] },
}

#[derive(Default)]
pub struct DebugState {
    elements: Vec<DebugElement>,
}

impl DebugState {

    pub fn any(&self) -> bool {
        self.elements.len() > 0
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn draw_rect(&mut self, rect: AABB, line_thickness: f32, color: [u8; 4]) {
        self.elements.push(DebugElement::Rect { base: rect, line_thickness, color });
    }

    /// Returns [index_count, index_buffer_size, vertex_buffer_size] required to hold the current debug state
    pub fn buffers_sizes(&self) -> [usize; 3] {
        let mut index_count = 0usize;
        let mut vertex_count = 0usize;
        for debug in self.elements.iter() {
            match debug {
                DebugElement::Rect { .. } => {
                    index_count += 24;
                    vertex_count += 8;
                }
            }
        }

        [
            index_count,
            crate::shared::align_up(index_count * size_of::<u16>(), 4),
            vertex_count * size_of::<GpuDebugVertex>(),
        ]
    }

    

    /// Generate the debug mesh. index_slice and vertex_slice must be large enough to contain the sizes returned by `buffers_sizes`
    /// Safety: `index_slice`` and `vertex_slice` must be aligned to 4 bytes
    pub fn generate_mesh(&self, index_slice: &mut [u8], vertex_slice: &mut [u8]) {
        let (_, index, _) = unsafe { index_slice.align_to_mut::<u16>() };
        let (_, vertex, _) = unsafe { vertex_slice.align_to_mut::<GpuDebugVertex>() };

        let mut state = GenerateMeshState {
            index_count: 0,
            vertex_count: 0,
            index,
            vertex
        };

        for &debug in self.elements.iter() {
            match debug {
                DebugElement::Rect { .. } => state.generate_rect(debug),
            }
        }
    }

}

struct GenerateMeshState<'a> {
    index_count: usize,
    vertex_count: usize,
    index: &'a mut [u16],
    vertex: &'a mut [GpuDebugVertex]
}

impl<'a> GenerateMeshState<'a> {

    fn generate_rect(&mut self, element: DebugElement) {
        let (base, t, color) = match element {
            DebugElement::Rect { base, line_thickness, color } => (base, line_thickness, color)
        };

        // 0-----4
        // | 1 5 |
        // | 3 7 |
        // 2-----6

        let i = self.index_count;
        let v = self.vertex_count as u16;
        self.index[i+0..i+6].copy_from_slice(&[v+0, v+5, v+4, v+0, v+1, v+5]);    // Top
        self.index[i+6..i+12].copy_from_slice(&[v+3, v+2, v+7, v+7, v+2, v+6]);   // Bottom
        self.index[i+12..i+18].copy_from_slice(&[v+0, v+2, v+1, v+1, v+2, v+3]);  // Left
        self.index[i+18..i+24].copy_from_slice(&[v+4, v+5, v+6, v+5, v+7, v+6]);  // Right

        let v = self.vertex_count;
        self.vertex[v+0] = GpuDebugVertex { position: [base.left, base.top]           , color };
        self.vertex[v+1] = GpuDebugVertex { position: [base.left + t, base.top + t]   , color };
        self.vertex[v+2] = GpuDebugVertex { position: [base.left, base.bottom]        , color };
        self.vertex[v+3] = GpuDebugVertex { position: [base.left + t, base.bottom - t], color };

        self.vertex[v+4] = GpuDebugVertex { position: [base.right, base.top]           , color };
        self.vertex[v+5] = GpuDebugVertex { position: [base.right - t, base.top + t]   , color };
        self.vertex[v+6] = GpuDebugVertex { position: [base.right, base.bottom]        , color };
        self.vertex[v+7] = GpuDebugVertex { position: [base.right - t, base.bottom - t], color };

        self.index_count += 24;
        self.vertex_count += 8;
    }

}

