use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::store::StoreLoad;

#[derive(Copy, Clone, TryFromBytes, IntoBytes, Immutable)]
#[repr(u8)]
pub enum TerrainCell {
    Grass,
}

pub struct Terrain {
    width: u32,
    height: u32,
    cells: Vec<TerrainCell>
}

impl Terrain {

    pub(super) fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.cells = vec![TerrainCell::Grass; (width*height) as usize];
    }

    pub const fn cell_count(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
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

