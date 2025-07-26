use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::shared::AABB_U32;
use crate::store::StoreLoad;


// This value is also hardcoded in the terrain shaders
pub const TERRAIN_SPRITE_SIZE: f32 = 64.0;

#[derive(Copy, Clone, TryFromBytes, IntoBytes, Immutable)]
#[repr(u8)]
pub enum TerrainCell {
    Water=0,
    Grass,
}

/// Bare minimum to build a 2D terrain
pub struct Terrain {
    width: u32,
    height: u32,
    cells: Vec<TerrainCell>
}

impl Terrain {

    pub(super) fn init(&mut self, width: u32, height: u32) {
        assert!(width > 2 && height > 2, "Terrain min size in 2x2");
        assert!(width <= 255 && height <= 255, "Terrain max size is 255x255");

        self.width = width;
        self.height = height;
        self.cells = vec![TerrainCell::Water; (width*height) as usize];
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn get_cell(&self, x_index: usize, y_index: usize) -> TerrainCell {
        let width = self.width as usize;
        let x_index = usize::min(x_index, width-1);
        let y_index = usize::min(y_index, (self.height as usize) - 1);
        let index = (y_index*width) + x_index;
        match self.cells.get(index) {
            Some(cell) => *cell,
            None => unsafe { std::hint::unreachable_unchecked() } // index will always be in range
        }
    }

    pub fn paint_rect(&mut self, cell_type: TerrainCell, mut rect: AABB_U32) {
        rect.right = u32::min(rect.right, self.width);
        rect.bottom = u32::min(rect.bottom, self.height);

        let width = self.width;

        for y in rect.top..rect.bottom {
            for x in rect.left..rect.right {
                let index = ((y*width) + x) as usize;
                self.cells[index] = cell_type;
            }
        }
    }

}

impl StoreLoad for Terrain {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.width);
        writer.write(&self.height);
        writer.write_array(&self.cells);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut terrain = Terrain::default();
        terrain.width = reader.try_read()?;
        terrain.height = reader.try_read()?;
        terrain.cells = unsafe { reader.read_array_transmute().to_vec() };
        Ok(terrain)
    }
}

impl Default for Terrain {
    fn default() -> Terrain {
        Terrain {
            width: 0,
            height: 0,
            cells: Vec::new()
        }
    }
}

