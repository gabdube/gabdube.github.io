use crate::data::{assets::TerrainSprites, terrain::{Terrain, TerrainCell, TERRAIN_SPRITE_SIZE}};
use super::gpu_shared::GpuTerrainSpriteData;

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

    pub fn cell_count(&self) -> usize {
        self.terrain.cell_count()
    }

    pub fn size_bytes(&self) -> usize {
        self.terrain.cell_count() * size_of::<GpuTerrainSpriteData>()
    }

    fn get_cell_uv(&self, cell: TerrainCell) -> [f32; 2] {
        match cell {
            TerrainCell::Grass => self.sprites.grass.offset,
            TerrainCell::Water => self.sprites.water.offset,
        }
    }

    pub fn generate_instances(&self, output: &mut [u8]) {
        let (_, instances, _) = unsafe { output.align_to_mut::<GpuTerrainSpriteData>() };
        let height = self.terrain.height() as usize;
        let width = self.terrain.width() as usize;

        let mut x = 0.0;
        let mut y = 0.0;
        for y_index in 0..height {
            for x_index in 0..width {
                let offset = (y_index*width) + x_index;
                let cell = self.terrain.get_cell(x_index, y_index);
                let position = [x, y];
                let uv = self.get_cell_uv(cell);
                instances[offset] = GpuTerrainSpriteData { position, uv };

                x += TERRAIN_SPRITE_SIZE;
            }

            x = 0.0;
            y += TERRAIN_SPRITE_SIZE;
        }

    }

}
