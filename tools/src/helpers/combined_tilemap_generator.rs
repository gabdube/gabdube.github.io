use crate::{helpers::PIXEL_SIZE, shared};
use super::SpriteData;

pub const TILE_TYPE_BACKGROUND: u16 = 1;
pub const TILE_TYPE_15_PIECES: u16 = 2;

/// A tilemap used with a dual grid system
pub struct Tilemap15Pieces {
    pub name: String,
    pub data: SpriteData,
    pub tile_size: u32,
}

impl Tilemap15Pieces {

    // Re-organize tiles so they can be rapidly indexed by an index
    pub fn remap(&mut self) {
        let mut remapped = SpriteData::empty_from_size(self.data.size);

        // [original_x, original_y, remapped_index]
        let tiles_remap: [[u8;3]; 16] = [
            [0, 0, 4],
            [1, 0, 10],
            [2, 0, 13],
            [3, 0, 12],
            [0, 1, 9],
            [1, 1, 14],
            [2, 1, 15],
            [3, 1, 7],
            [0, 2, 2],
            [1, 2, 3],
            [2, 2, 11],
            [3, 2, 5],
            [0, 3, 0],
            [1, 3, 8],
            [2, 3, 6],
            [3, 3, 1],
        ];

        let ts = self.tile_size;

        for [x, y, remapped_index] in tiles_remap {
            let x = x as u32 * ts;
            let y = y as u32 * ts;
            let remapped_x = (remapped_index & 0b11) as u32 * ts;
            let remapped_y = (remapped_index >> 2) as u32 * ts;

            let src = shared::RectU32 {
                left: x,
                top: y,
                right: x + ts,
                bottom: y + ts,
            };

            let dst = shared::RectU32 {
                left: remapped_x,
                top: remapped_y,
                right: remapped_x + ts,
                bottom: remapped_y + ts,
            };

            self.data.copy_pixels(&mut remapped, src, dst);
        }


        self.data = remapped;
    }

}

/// A single "background" sprite
pub struct BackgroundTile {
    pub name: String,
    pub data: SpriteData,
}

impl BackgroundTile {
    pub fn new(name: String, data: SpriteData) -> Self {
        BackgroundTile { name, data }
    }
}

/// Different typemap types that can be added to a [CombinedTilemap]
pub enum InputTilemapTypes {
    Background(BackgroundTile),
    Tilemap15Pieces(Tilemap15Pieces),
}

#[derive(Copy, Clone)]
pub struct TileMapping {
    pub offset_x: u32,
    pub offset_y: u32,
    pub ty: u16,
    pub data_index: u16,
}

/// Generated tilemap
pub struct CombinedTilemap {
    pub backgrounds: Vec<BackgroundTile>,
    pub tiles15pieces: Vec<Tilemap15Pieces>,
    pub tile_mapping: Vec<TileMapping>,
    pub output_image_pixels: Vec<u8>,
    pub output_image_size: shared::SizeU32,
    pub tile_size: u32,
}

impl CombinedTilemap {

    pub fn add_tilemap(&mut self, input_tilemap: InputTilemapTypes) -> Result<(), String> {
        let mut error = None;

        match input_tilemap {
            InputTilemapTypes::Background(bg) => {
                let tile_size = bg.data.frame_size.width;
                if self.tile_size != 0 && self.tile_size != tile_size {
                    error = Some(format!("Mismatching tile size for {:?}. Old: {}, New: {}", bg.name, self.tile_size, tile_size))
                } else {
                    self.tile_size = tile_size;
                    self.backgrounds.push(bg);
                }
            },
            InputTilemapTypes::Tilemap15Pieces(tiles) => {
                if self.tile_size != 0 && self.tile_size != tiles.tile_size {
                    error = Some(format!("Mismatching tile size or {:?}. Old: {}, New: {}", tiles.name, self.tile_size, tiles.tile_size))
                } else {
                    self.tile_size = tiles.tile_size;
                    self.tiles15pieces.push(tiles);
                }
            }
        }

        if let Some(err) = error {
            return Err(err);
        }

        Ok(())
    }

    fn compute_output_image_size(&mut self) {
        let width = self.tile_size * 4;
        let mut height = 0;

        let background_count = shared::align_up(self.backgrounds.len(), 4) as u32;
        height += self.tile_size * (background_count / 4);
        height += self.tile_size * ((self.tiles15pieces.len() * 4) as u32);

        self.output_image_size = shared::size_u32(width, height);

        let total_pixel_size = (width as usize) * (height as usize) * PIXEL_SIZE;
        self.output_image_pixels = vec![0; total_pixel_size];
    }

    fn compute_mapping(&mut self) {
        let ts = self.tile_size;
        let mut offset_x = 0;
        let mut offset_y = 0;

        // Adds background first
        for i in 0..self.backgrounds.len() {
            self.tile_mapping.push(TileMapping { 
                ty: TILE_TYPE_BACKGROUND,
                data_index: i as u16,
                offset_x,
                offset_y
            });
            offset_x += ts;
            if offset_x >= self.output_image_size.width {
                offset_x = 0;
                offset_y += ts;
            }
        }

        if offset_x > 0 {
            offset_y += ts;
        }

        // Then add 15 pieces tile maps
        for i in 0..self.tiles15pieces.len() {
            self.tile_mapping.push(TileMapping {
                ty: TILE_TYPE_15_PIECES,
                offset_x: 0,
                offset_y: offset_y,
                data_index: i as u16
            });
            offset_y += ts*4;
        }
    }

    fn remap_tiles(&mut self) {
        for tilemap in self.tiles15pieces.iter_mut() {
            tilemap.remap();
        }
    }

    fn copy_tiles(&mut self) {
        let dst_stride = self.output_image_size.width as usize * PIXEL_SIZE;
        let dst_bytes = &mut self.output_image_pixels;

        for &mapping in self.tile_mapping.iter() {
            let [dst_x, dst_y] = [mapping.offset_x as usize, mapping.offset_y as usize];
            let (pixels, src_stride, height) = match mapping.ty {
                TILE_TYPE_BACKGROUND => {
                    let bg = &self.backgrounds[mapping.data_index as usize];
                    let src_stride = bg.data.frame_size.width as usize * PIXEL_SIZE;
                    let height = bg.data.frame_size.height as usize;

                    (&bg.data.pixels, src_stride, height)
                },
                TILE_TYPE_15_PIECES => {
                    let tiles = &self.tiles15pieces[mapping.data_index as usize];
                    let src_stride = tiles.data.frame_size.width as usize * PIXEL_SIZE;
                    let height = tiles.data.frame_size.height as usize;

                    (&tiles.data.pixels, src_stride, height)
                }
                _ => panic!("Unknown mapping type {}", mapping.ty)
            };

            shared::copy_sprite(
                dst_bytes, dst_x, dst_y, dst_stride,
                pixels, src_stride, height,
                PIXEL_SIZE,
            );
        }
    }

    pub fn process(&mut self) {
        self.compute_output_image_size();
        self.compute_mapping();
        self.remap_tiles();
        self.copy_tiles();
    }

    pub fn generate_csv(&self) -> String {
        let mut output = String::with_capacity(2000);

        output.push_str("GLOBALS;\n");
        output.push_str(&format!("{};{};{};\n", self.tile_size, self.output_image_size.width, self.output_image_size.height));

        output.push_str("TILEMAPS;\n");
        for mapping in self.tile_mapping.iter() {
            let name;
            let tilemap_type;
        
            match mapping.ty {
                TILE_TYPE_BACKGROUND => { 
                    tilemap_type = "background";
                    name = &self.backgrounds[mapping.data_index as usize].name; },
                TILE_TYPE_15_PIECES => {
                    tilemap_type = "15pieces";
                    name = &self.tiles15pieces[mapping.data_index as usize].name;
                },
                _ => unreachable!()
            };

            output.push_str(&format!("{};{};{};{};\n", name, tilemap_type, mapping.offset_x, mapping.offset_y));
        }

        output
    }

}

impl Default for CombinedTilemap {
    fn default() -> Self {
        CombinedTilemap {
            backgrounds: Vec::with_capacity(4),
            tiles15pieces: Vec::with_capacity(4),
            tile_mapping: Vec::with_capacity(32),
            output_image_pixels: Vec::new(),
            output_image_size: shared::SizeU32::default(),
            tile_size: 0,
        }
    }
}