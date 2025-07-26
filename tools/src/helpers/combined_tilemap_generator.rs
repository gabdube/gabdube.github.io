use crate::{helpers::PIXEL_SIZE, shared};
use super::{SpriteData, LoadSpriteParams};

pub const TILE_TYPE_BACKGROUND: u16 = 1;
pub const TILE_TYPE_DUAL: u16 = 2;

/// A tilemap in the "reduced" format
pub struct DualTilemap {
    pub name: String,
    pub data: SpriteData,
    pub tile_size: u32,
}

impl DualTilemap {

    pub fn extract_background(&self) -> BackgroundTile {
        let ts = self.tile_size;
        let [offset_left, offset_top] = [2*ts, 1*ts];

        let crop = LoadSpriteParams::crop(offset_left, offset_top, offset_left+ts, offset_top+ts);
        let data = SpriteData::load_from_sprite_data(&self.data, crop, 0);

        BackgroundTile {
            name: self.name.clone(),
            data,
        }
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
    Dual(DualTilemap),
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
    pub dual: Vec<DualTilemap>,
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
            InputTilemapTypes::Dual(dual) => {
                if self.tile_size != 0 && self.tile_size != dual.tile_size {
                    error = Some(format!("Mismatching tile size or {:?}. Old: {}, New: {}", dual.name, self.tile_size, dual.tile_size))
                } else {
                    self.tile_size = dual.tile_size;
                    self.backgrounds.push(dual.extract_background());
                    self.dual.push(dual);
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
        height += self.tile_size * ((self.dual.len() * 4) as u32);

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

        // Then add dual tile maps
        for i in 0..self.dual.len() {
            self.tile_mapping.push(TileMapping {
                ty: TILE_TYPE_DUAL,
                offset_x: 0,
                offset_y: offset_y,
                data_index: i as u16
            });
            offset_y += ts*4;
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
                TILE_TYPE_DUAL => {
                    let dual = &self.dual[mapping.data_index as usize];
                    let src_stride = dual.data.frame_size.width as usize * PIXEL_SIZE;
                    let height = dual.data.frame_size.height as usize;

                    (&dual.data.pixels, src_stride, height)
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
        self.copy_tiles();
    }

    pub fn generate_csv(&self) -> String {
        let mut output = String::with_capacity(2000);

        output.push_str("GLOBALS;\n");
        output.push_str(&format!("{};{};{};\n", self.tile_size, self.output_image_size.width, self.output_image_size.height));

        output.push_str("BACKGROUNDS;\n");
        for mapping in self.tile_mapping.iter().filter(|map| map.ty == TILE_TYPE_BACKGROUND ) {
            let name = &self.backgrounds[mapping.data_index as usize].name;
            output.push_str(&format!("{};{};{};\n", name, mapping.offset_x, mapping.offset_y));
        }

        output.push_str("FOREGROUNDS;\n");

        output
    }

}

impl Default for CombinedTilemap {
    fn default() -> Self {
        CombinedTilemap {
            backgrounds: Vec::with_capacity(4),
            dual: Vec::with_capacity(4),
            tile_mapping: Vec::with_capacity(32),
            output_image_pixels: Vec::new(),
            output_image_size: shared::SizeU32::default(),
            tile_size: 0,
        }
    }
}