#![allow(dead_code)]

use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::shared::{AABB, PositionF32};

const MAX_LAYER: usize = 2;

#[derive(Copy, Clone, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum DebugElement {
    Point { pt: PositionF32, size: f32, color: [u8; 4], _padding: [u8; 12] },
    Line { p0: PositionF32, p1: PositionF32, color: [u8; 4], _padding: [u8; 8] },
    Rect { base: AABB, line_thickness: f32, color: [u8; 4], _padding: [u8; 4] },
    FillRect { base: AABB, color: [u8; 4], _padding: [u8; 8] },
    Triangle { v0: PositionF32, v1: PositionF32, v2: PositionF32, color: [u8; 4] },
    FillTriangle { v0: PositionF32, v1: PositionF32, v2: PositionF32, color: [u8; 4] },
}

pub struct DebugState {
    /// Current layer in which the next elements will be added
    pub current_layer: usize,

    /// Collections of debug elements. One for each layer
    /// Elements at collection 0 are rendered first, then the others are renderered on top
    pub layers: Vec<Vec<DebugElement>>,
}

impl DebugState {

    pub fn any(&self) -> bool {
        self.layers.iter().any(|layer| !layer.is_empty() )
    }

    pub fn set_current_layer(&mut self, layer: usize) {
        assert!(layer < MAX_LAYER, "There should be no need to have more than two layers for now");
        self.current_layer = layer;
    }

    pub fn clear(&mut self) {
        self.current_layer = 0;
        for layer in self.layers.iter_mut() {
            layer.clear();
        }
    }

    pub fn draw_rect(&mut self, rect: AABB, line_thickness: f32, color: [u8; 4]) {
        self.layers[self.current_layer].push(DebugElement::Rect { base: rect, line_thickness, color, _padding: [0; 4] });
    }

    pub fn draw_triangle(&mut self, v0: PositionF32, v1: PositionF32, v2: PositionF32, color: [u8; 4]) {
        self.layers[self.current_layer].push(DebugElement::Triangle { v0, v1, v2, color });
    }

    pub fn fill_triangle(&mut self, v0: PositionF32, v1: PositionF32, v2: PositionF32, color: [u8; 4]) {
        self.layers[self.current_layer].push(DebugElement::FillTriangle { v0, v1, v2, color });
    }

    pub fn draw_point(&mut self, pt: PositionF32, size: f32, color: [u8; 4]) {
        self.layers[self.current_layer].push(DebugElement::Point { pt, size, color, _padding: [0; 12] });
    }

    pub fn draw_line(&mut self, p0: PositionF32, p1: PositionF32, color: [u8; 4]) {
        self.layers[self.current_layer].push(DebugElement::Line { p0, p1, color, _padding: [0; 8] });
    }

}



//
// Other impls
//

impl Default for DebugState {
    fn default() -> Self {
        let mut layers = Vec::new();
        for _ in 0..MAX_LAYER {
            layers.push(Vec::new());
        }

        DebugState { current_layer: 0, layers }
    }
}