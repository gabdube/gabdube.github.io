use crate::data::assets::{TerrainSprites, Terrain15PiecesMask};
use crate::data::terrain::{Terrain, BackgroundCell, ForegroundCell};
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

    pub const fn max_cell_count(&self) -> usize {
        let [width, height] = self.terrain.inner_size().splat();
        (width as usize) * (height as usize) * 4
    }

    pub fn max_size_bytes(&self) -> usize {
        self.max_cell_count() * size_of::<GpuTerrainSpriteData>()
    }

    fn mask_for_tilemap15(base: BackgroundCell, background: [BackgroundCell; 4]) -> Terrain15PiecesMask {
        let mut mask = Terrain15PiecesMask::default();
        if base == background[0] { mask |= Terrain15PiecesMask::TOP_LEFT; }
        if base == background[1] { mask |= Terrain15PiecesMask::TOP_RIGHT; }
        if base == background[2] { mask |= Terrain15PiecesMask::BOTTOM_LEFT; }
        if base == background[3] { mask |= Terrain15PiecesMask::BOTTOM_RIGHT; }
        mask
    }

    fn get_cell_texcoord(&self, cell: ForegroundCell, base: BackgroundCell) -> [u8; 2] {
        match base {
            BackgroundCell::Water => self.sprites.water.base_offset(),
            BackgroundCell::Grass => self.sprites.grass.get_offset_from_mask(Self::mask_for_tilemap15(base, cell.background)),
            BackgroundCell::Last => self.sprites.missing.base_offset(),
        }
    }

    pub fn generate_instances(&self, output: &mut [u8]) -> usize {
        let (_, instances, _) = unsafe { output.align_to_mut::<GpuTerrainSpriteData>() };
        let mut offset = 0;
        for cell in self.terrain.inner_cells() {
            let [x, y] = cell.position;
            let (count, unique) = cell.unique_background_cells();

            for &background in unique[0..count].iter() {
                let [tx, ty] = self.get_cell_texcoord(cell, background);
                let data = (x as u32) << 24 | (y as u32) << 16 | (tx as u32) << 8 | (ty as u32);
                instances[offset] = GpuTerrainSpriteData { data };
                offset += 1;
            }
        }

        offset
    }

}
