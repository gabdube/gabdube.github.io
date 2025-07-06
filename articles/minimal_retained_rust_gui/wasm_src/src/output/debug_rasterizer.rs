use crate::data::debug::{DebugState, DebugElement};
use crate::shared::{PositionF32, AABB};
use super::gpu_shared::GpuDebugVertex;

pub(super) struct DebugMeshRasterizer<'a> {
    index_count: usize,
    vertex_count: usize,
    index: &'a mut [u16],
    vertex: &'a mut [GpuDebugVertex]
}

impl<'a> DebugMeshRasterizer<'a> {

    /// Returns [index_count, index_buffer_size, vertex_buffer_size] required to hold the current debug state
    pub fn buffers_sizes(state: &DebugState) -> [usize; 3] {
        let mut index_count = 0usize;
        let mut vertex_count = 0usize;

        for layer in state.layers.iter() {
            for debug in layer.iter() {
                match debug {
                    DebugElement::Rect { .. } => {
                        index_count += 24;
                        vertex_count += 8;
                    },
                    DebugElement::FillRect { .. } | DebugElement::Point { .. } | DebugElement::Line { .. } => {
                        index_count += 6;
                        vertex_count += 4;
                    },
                    DebugElement::Triangle { .. } => {
                        index_count += 18;
                        vertex_count += 12;
                    },
                    DebugElement::FillTriangle { .. } => {
                        index_count += 3;
                        vertex_count += 3;
                    }
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
    pub fn generate_mesh(state: &DebugState, index_slice: &mut [u8], vertex_slice: &mut [u8]) {
        let (_, index, _) = unsafe { index_slice.align_to_mut::<u16>() };
        let (_, vertex, _) = unsafe { vertex_slice.align_to_mut::<GpuDebugVertex>() };

        let mut rasterizer = DebugMeshRasterizer {
            index_count: 0,
            vertex_count: 0,
            index,
            vertex
        };
        for layer in state.layers.iter() {
            for &debug in layer.iter() {
                match debug {
                    DebugElement::Point { ..  }=> rasterizer.generate_point(debug),
                    DebugElement::Line { ..  }=> rasterizer.generate_line(debug),
                    DebugElement::Rect { .. } => rasterizer.generate_rect(debug),
                    DebugElement::FillRect { .. } => rasterizer.generate_fill_rect(debug),
                    DebugElement::Triangle { .. } => rasterizer.generate_triangle(debug),
                    DebugElement::FillTriangle { .. } => rasterizer.generate_fill_triangle(debug),
                }
            }
        }
    }

    fn generate_rect(&mut self, element: DebugElement) {
        let (base, t, color) = match element {
            DebugElement::Rect { base, line_thickness, color, .. } => (base, line_thickness, color),
            _ => unsafe { ::std::hint::unreachable_unchecked() }
        };

        // 0-----4
        // | 1 5 |
        // | 3 7 |
        // 2-----6

        let i = self.index_count;
        let v = self.vertex_count as u16;

        assert!(self.index.len() >= i+24, "Index buffer is not large enough");
        assert!(self.vertex.len() >= (v as usize)+8, "Vertex buffer is not large enough");

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

    fn generate_fill_rect(&mut self, element: DebugElement) {
        let (base, color) = match element {
            DebugElement::FillRect { base, color, .. } => (base, color),
            _ => unsafe { ::std::hint::unreachable_unchecked() }
        };

        let i = self.index_count;
        let v = self.vertex_count as u16;

        assert!(self.index.len() >= i+6, "Index buffer is not large enough");
        assert!(self.vertex.len() >= (v as usize)+4, "Vertex buffer is not large enough");

        self.index[i+0..i+6].copy_from_slice(&[v+0, v+1, v+2, v+0, v+2, v+3]);

        let v = self.vertex_count;
        let [left, top, right, bottom] = base.splat();
        self.vertex[v+0] = GpuDebugVertex { position: [left, top]    , color };
        self.vertex[v+1] = GpuDebugVertex { position: [left, bottom] , color };
        self.vertex[v+2] = GpuDebugVertex { position: [right, bottom]   , color };
        self.vertex[v+3] = GpuDebugVertex { position: [right, top], color };

        self.index_count += 6;
        self.vertex_count += 4;
    }

    fn generate_triangle(&mut self, element: DebugElement) {
        let (v0, v1, v2, color) = match element {
            DebugElement::Triangle { v0, v1, v2, color } => (v0, v1, v2, color),
            _ => unsafe { ::std::hint::unreachable_unchecked() }
        };

        self.generate_line_inner(v0, v1, color);
        self.generate_line_inner(v1, v2, color);
        self.generate_line_inner(v0, v2, color);
    }

    fn generate_fill_triangle(&mut self, element: DebugElement) {
        let (v0, v1, v2, color) = match element {
            DebugElement::FillTriangle { v0, v1, v2, color } => (v0, v1, v2, color),
            _ => unsafe { ::std::hint::unreachable_unchecked() }
        };

        let i = self.index_count;
        let v = self.vertex_count as u16;
        
        assert!(self.index.len() >= i+3, "Index buffer is not large enough");
        assert!(self.vertex.len() >= (v as usize)+3, "Vertex buffer is not large enough");

        self.index[i+0..i+3].copy_from_slice(&[v+0, v+1, v+2]);
        
        let v = self.vertex_count;
        self.vertex[v+0] = GpuDebugVertex { position: v0.splat(), color };
        self.vertex[v+1] = GpuDebugVertex { position: v1.splat(), color };
        self.vertex[v+2] = GpuDebugVertex { position: v2.splat(), color };

        self.index_count += 3;
        self.vertex_count += 3;
    }

    fn generate_point(&mut self, element: DebugElement) {
        let (pt, size, color) = match element {
            DebugElement::Point { pt, size, color, .. } => (pt, size, color),
            _ => unsafe { ::std::hint::unreachable_unchecked() }
        };
        
        let base = AABB { left: pt.x - size, right: pt.x + size, top: pt.y - size, bottom: pt.y + size };
        let fill = DebugElement::FillRect { base, color: color, _padding: [0; 8] };
        self.generate_fill_rect(fill);
    }

    fn generate_line(&mut self, element: DebugElement) {
        let (p0, p1, color) = match element {
            DebugElement::Line { p0, p1, color, .. } => (p0, p1, color),
            _ => unsafe { ::std::hint::unreachable_unchecked() }
        };

        self.generate_line_inner(p0, p1, color);
    }

    fn generate_line_inner(&mut self, p0: PositionF32, p1: PositionF32, color: [u8; 4]) {
        const LINE_WIDTH: f32 = 1.0;

        let angle = f32::atan2(p1.y-p0.y, p1.x-p0.x);
        let y = LINE_WIDTH * f32::cos(angle);
        let x = LINE_WIDTH* f32::sin(angle);

        let i = self.index_count;
        let v = self.vertex_count as u16;

        assert!(self.index.len() >= i+6, "Index buffer is not large enough");
        assert!(self.vertex.len() >= (v as usize)+4, "Vertex buffer is not large enough");

        self.index[i+0..i+6].copy_from_slice(&[v+0, v+1, v+2, v+0, v+2, v+3]);

        let v = self.vertex_count;
        self.vertex[v+0] = GpuDebugVertex { position: [p0.x+x, p0.y-y],  color };
        self.vertex[v+1] = GpuDebugVertex { position: [p0.x-x, p0.y+y],  color };
        self.vertex[v+2] = GpuDebugVertex { position: [p1.x-x, p1.y+y],  color };
        self.vertex[v+3] = GpuDebugVertex { position: [p1.x+x, p1.y-y],  color };

        self.index_count += 6;
        self.vertex_count += 4;
    }

}
