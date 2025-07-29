use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::shared::parse_u32;

#[derive(Default, Copy, Clone, Debug)]
pub struct Terrain15PiecesMask(pub u8);
impl Terrain15PiecesMask {
    pub const TOP_LEFT: Self = Self(0x1);
    pub const TOP_RIGHT: Self = Self(0x2);
    pub const BOTTOM_LEFT: Self = Self(0x4);
    pub const BOTTOM_RIGHT: Self = Self(0x8);
}

impl ::std::ops::BitOrAssign for Terrain15PiecesMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// A single tile
#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
#[repr(align(4))]
pub struct TerrainBackgroundSprite {
    /// Offset in tiles (only first two values are used, padded to 4 bytes for storage purpose)
    pub offset: [u8; 4],
}

impl TerrainBackgroundSprite {
    #[inline(always)]
    pub const fn base_offset(&self) -> [u8; 2] {
        [self.offset[0], self.offset[1]]
    }
}

/// A set of 15 tiles. Indexed using [Terrain15PieceMask]
#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
#[repr(align(4))]
pub struct Terrain15Pieces {
    /// Offset in tiles (only first two values are used, padded to 4 bytes for storage purpose)
    pub base_offset: [u8; 4],
}

impl Terrain15Pieces {

    #[inline(always)]
    pub const fn get_offset_from_mask(&self, mask: Terrain15PiecesMask) -> [u8; 2] {
        let [base_x, base_y, _, _] = self.base_offset;
        let x = mask.0 & 0b11;
        let y = mask.0 >> 2;
        [base_x + x, base_y + y]
    }

}


#[derive(Default)]
pub struct TerrainSprites {
    pub tile_size: u32,
    pub tileset_width: u32,
    pub tileset_height: u32,
    pub water: TerrainBackgroundSprite,
    pub missing: TerrainBackgroundSprite,
    pub grass: Terrain15Pieces
}

impl TerrainSprites {
    fn load_background(&mut self, name: &str, offset: [u8; 4]) {
        match name {
            "water" => { self.water = TerrainBackgroundSprite { offset }; },
            "missing" => { self.missing = TerrainBackgroundSprite { offset }; },
            _ => {
                warn!("Invalid background sprite name {:?}", name);
            }
        }
    }

    fn load_15pieces(&mut self, name: &str, base_offset: [u8; 4]) {
        match name {
            "grass" => { self.grass = Terrain15Pieces { base_offset }; },
            _ => {
                warn!("Invalid background sprite name {:?}", name);
            }
        }
    }

    pub fn load_from_csv(&mut self, csv: &str) {
        const LOAD_GLOBALS: u32 = 1;
        const LOAD_TILEMAPS: u32 = 2;

        let mut state = 0;

        crate::shared::split_csv::<4, _>(csv, |args| {
            let first_arg = *args.get(0).unwrap_or(&"");
            match first_arg {
                "GLOBALS" => { state = LOAD_GLOBALS; return; }
                "TILEMAPS" => { state = LOAD_TILEMAPS; return; }
                _ => {}
            }

            match state {
                LOAD_GLOBALS => {
                    self.tile_size =  parse_u32(args.get(0));
                    self.tileset_width = parse_u32(args.get(1));
                    self.tileset_height = parse_u32(args.get(2));
                    if self.tile_size == 0 || self.tileset_width == 0 || self.tileset_height == 0 {
                        warn!("Invalid terrain global state {:?}", args);
                    }
                },
                LOAD_TILEMAPS => {
                    if args.len() < 4 {
                        warn!("Invalid arguments count for tilemaps (expected 4) {:?}", args);
                        return;
                    }

                    let name = args[0];
                    let tilemap_type = args[1];

                    let x = parse_u32(args.get(2)) / self.tile_size;
                    let y = parse_u32(args.get(3)) / self.tile_size;
                    let offset = [x as u8, y as u8, 0, 0];

                    match tilemap_type {
                        "background" => self.load_background(name, offset),
                        "15pieces" => self.load_15pieces(name, offset),
                        _ => {
                            warn!("Unknown tilemap type {:?}", tilemap_type);
                            return;
                        }
                    }
                },
                _ => { warn!("Unknown state while loading terrain sprites {:?}", state) }
            }
        });
    }
}
