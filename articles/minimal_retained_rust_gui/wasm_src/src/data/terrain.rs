use zerocopy_derive::{Immutable, IntoBytes, TryFromBytes};
use crate::shared::{AABB_U32, SizeU32, size_u32};
use crate::store::StoreLoad;


// This value is also hardcoded in the terrain shaders
pub const TERRAIN_SPRITE_SIZE: f32 = 64.0;

#[derive(Debug, Copy, Clone, TryFromBytes, IntoBytes, Immutable, PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum BackgroundCell {
    Water=0,
    Grass,
    Last=255,
}

#[derive(Copy, Clone)]
#[repr(align(4))]
pub struct ForegroundCell {
    pub position: [u8; 2],
    pub background: [BackgroundCell; 4],
}

impl ForegroundCell {

    /// Sort the 4 terrain cells from top to bottom in the order they should be rendered
    /// In the [TerrainCell] type, higher cell number must be rendered on top of the lower numbers
    pub fn rendering_sorted(&self) -> [BackgroundCell; 4] {
        let mut sorted = self.background;
        if sorted[0] > sorted[1] { sorted.swap(0, 1); }
        if sorted[2] > sorted[3] { sorted.swap(2, 3); }
        if sorted[0] > sorted[2] { sorted.swap(0, 2); }
        if sorted[1] > sorted[3] { sorted.swap(1, 3); }
        if sorted[1] > sorted[2] { sorted.swap(1, 2); }
        if sorted[0] > sorted[1] { sorted.swap(0, 1); }
        sorted
    }

    /// Count and return the unique background cells in this foreground cell
    /// Cells are returned in the order they must be rendered
    pub fn unique_background_cells(&self) -> (usize, [BackgroundCell; 4]) {
        let sorted = self.rendering_sorted();

        let mut count = 0;
        let mut unique = [BackgroundCell::Last; 4];

        unique[0] = sorted[0];
        count += 1;

        if sorted[1] != unique[count-1] {
            unique[count] = sorted[1];
            count += 1;
        }

        if sorted[2] != unique[count-1] {
            unique[count] = sorted[2];
            count += 1;
        }

        if sorted[3] != unique[count-1] {
            unique[count] = sorted[3];
            count += 1;
        }

        (count, unique)
    }

}

/// Bare minimum to build a 2D terrain
pub struct Terrain {
    width: u32,
    height: u32,
    background_cells: Vec<BackgroundCell>
}

impl Terrain {

    pub(super) fn init(&mut self, width: u32, height: u32) {
        assert!(width > 2 && height > 2, "Terrain min size in 2x2");
        assert!(width <= 255 && height <= 255, "Terrain max size is 255x255");

        self.width = width;
        self.height = height;
        self.background_cells = vec![BackgroundCell::Water; (width*height) as usize];
    }

    pub const fn size(&self) -> SizeU32 {
        size_u32(self.width , self.height)
    }

    pub const fn inner_size(&self) -> SizeU32 {
        size_u32(self.width - 1, self.height - 1)
    }

    fn get_inner_cell(&self, x: usize, y: usize) -> [BackgroundCell; 4] {
        let bg = &self.background_cells;
        let width = self.width as usize;
        let i1 = (y * width) + x;
        let i2 = i1 + 1;
        let i3 = i1 + width;
        let i4 = i2 + width;

        match [bg.get(i1), bg.get(i2), bg.get(i3), bg.get(i4)] {
            [Some(c1), Some(c2), Some(c3), Some(c4)] => [*c1, *c2, *c3, *c4],
            _ => unsafe { ::std::hint::unreachable_unchecked(); } // x and y will always be within the terrain bounds
        }
    }

    pub fn inner_cells<'a>(&'a self) -> impl Iterator<Item=ForegroundCell> + 'a {
        let [inner_width, inner_height] = self.inner_size().splat();
        let [inner_width, inner_height] = [inner_width as usize, inner_height as usize];
        let [mut x, mut y] = [0, 0];
        return std::iter::from_fn(move || {
            if y == inner_height {
                return None;
            }

            let cell = ForegroundCell {
                position: [x as u8, y as u8],
                background: self.get_inner_cell(x, y)
            };

            x += 1;
            if x == inner_width {
                x = 0;
                y += 1;
            }

            Some(cell)
        });
    }

    pub fn paint_rect(&mut self, cell_type: BackgroundCell, mut rect: AABB_U32) {
        rect.right = u32::min(rect.right, self.width);
        rect.bottom = u32::min(rect.bottom, self.height);

        let width = self.width;

        for y in rect.top..rect.bottom {
            for x in rect.left..rect.right {
                let index = ((y*width) + x) as usize;
                self.background_cells[index] = cell_type;
            }
        }
    }

}

impl StoreLoad for Terrain {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.width);
        writer.write(&self.height);
        writer.write_array(&self.background_cells);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut terrain = Terrain::default();
        terrain.width = reader.try_read()?;
        terrain.height = reader.try_read()?;
        terrain.background_cells = unsafe { reader.read_array_transmute().to_vec() };
        Ok(terrain)
    }
}

impl Default for Terrain {
    fn default() -> Terrain {
        Terrain {
            width: 0,
            height: 0,
            background_cells: Vec::new()
        }
    }
}

