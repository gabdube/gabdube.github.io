use crate::data::{assets::TerrainSprites, terrain::{Terrain, TerrainCell}};
use super::gpu_shared::GpuTerrainSpriteData;

/**
    The terrain is rendered using two grid. One background grid and one foreground grid.
*/
pub(super) struct TerrainMeshRasterizer<'a> {
    sprites: &'a TerrainSprites,
    terrain: &'a Terrain,
}

impl<'a> TerrainMeshRasterizer<'a> {

    pub fn new(sprites: &'a TerrainSprites, terrain: &'a Terrain) -> Self {
        TerrainMeshRasterizer {
            sprites,
            terrain,
        }
    }

    pub const fn background_cell_count(&self) -> usize {
        let width = self.terrain.width() as usize;
        let height = self.terrain.height() as usize;
        width * height
    }

    pub const fn foreground_cell_count(&self) -> usize {
        let width = (self.terrain.width() - 1) as usize;
        let height = (self.terrain.height() - 1) as usize;
        width * height
    }

    pub const fn cell_count(&self) -> usize {
        self.background_cell_count() + self.foreground_cell_count()
    }

    pub fn size_bytes(&self) -> usize {
        self.cell_count() * size_of::<GpuTerrainSpriteData>()
    }

    fn get_cell_uv(&self, x: usize, y: usize) -> [u8; 2] {
        let [x, y, _, _] = match self.terrain.get_cell(x, y) {
            TerrainCell::Grass => self.sprites.grass.offset,
            TerrainCell::Water => self.sprites.water.offset,
        };

        [x, y]
    }

    pub fn generate_instances(&self, output: &mut [u8]) {
        self.generate_background_instances(output);
        self.generate_foreground_instances(output);
    }

    pub fn generate_background_instances(&self, output: &mut [u8]) {
        let (_, instances, _) = unsafe { output.align_to_mut::<GpuTerrainSpriteData>() };
        let background_instance_offset = 0;
        let height = self.terrain.height() as usize;
        let width = self.terrain.width() as usize;

        for y in 0..height {
            for x in 0..width {
                let offset = (y * width) + x;
                let [uv_x, uv_y] = self.get_cell_uv(x, y);

                let mut data = 0u32;
                data += (x as u32) << 24;
                data += (y as u32) << 16;
                data += (uv_x as u32) << 8;
                data += uv_y as u32;

                instances[background_instance_offset + offset] = GpuTerrainSpriteData { data };
            }
        }
    }

    pub fn generate_foreground_instances(&self, output: &mut [u8]) {
        let (_, instances, _) = unsafe { output.align_to_mut::<GpuTerrainSpriteData>() };
        let background_instance_offset = self.background_cell_count();
        let height = self.terrain.height() as usize - 1;
        let width = self.terrain.width() as usize - 1;

        for y in 0..height {
            for x in 0..width {
                let offset = (y * width) + x;
                let mut data = 0u32;
                data += (x as u32) << 24;
                data += (y as u32) << 16;
                data += (1 as u32) << 8;
                data += 0 as u32;

                instances[background_instance_offset + offset] = GpuTerrainSpriteData { data };
            }
        }
    }

}
