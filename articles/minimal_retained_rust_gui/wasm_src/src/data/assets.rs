mod tilemaps;
pub use tilemaps::{TerrainSprites, Terrain15PiecesMask};

use fnv::FnvHashMap;
use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::error::Error;
use crate::shared::{AABB, parse_f32, parse_u32};
use crate::store::StoreLoad;
use crate::GameClientInit;
use super::sprites::{StaticSprite, AnimatedSprite};

#[derive(Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct Texture {
    // The unique ID of the texture that identify the resource on the engine side
    pub id: u32,
}

#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
pub struct AtlasData {
    pub texture: Texture,
    pub castle: StaticSprite,
    pub house: StaticSprite,
    pub tower: StaticSprite,

    pub knight_idle: AnimatedSprite,
    pub knight_run: AnimatedSprite,
}

impl AtlasData {
    fn load_from_csv(&mut self, csv: &str) {
        crate::shared::split_csv::<7, _>(csv, |args| {
            let name = args[0];
            let frame_count = parse_u32(args.get(1));
            let left = parse_f32(args.get(2));
            let top = parse_f32(args.get(3));
            let right = parse_f32(args.get(4));
            let bottom = parse_f32(args.get(5));

            match name {
                "castle" => { self.castle = StaticSprite { texcoord: AABB { left, top, right, bottom } }; }
                "house" => { self.house = StaticSprite { texcoord: AABB { left, top, right, bottom } }; }
                "tower" => { self.tower = StaticSprite { texcoord: AABB { left, top, right, bottom } }; }

                "warrior_idle" => { self.knight_idle = AnimatedSprite { sprite_base: AABB { left, top, right, bottom }, frame_count }; }
                "warrior_run" => { self.knight_run = AnimatedSprite { sprite_base: AABB { left, top, right, bottom }, frame_count }; }

                _ => { warn!("Unknown atlas key {:?}", name) }
            }
        });
    }
}


pub struct Assets {
    pub textures: FnvHashMap<String, Texture>,
    pub atlas: AtlasData,
    pub terrain: TerrainSprites,
}

impl Assets {

    pub fn init(&mut self, init: &GameClientInit) -> Result<(), Error> {
        self.import_assets_index(init)?;

        self.atlas.texture = self.textures.get("atlas")
            .copied()
            .ok_or_else(|| assets_err!("Missing texture \"atlas\" ") )?;

        Ok(())
    }

    fn load_texture(&mut self, args: &[&str]) -> Result<(), Error> {
        let name = args.get(1)
            .map(|value| value.to_string() )
            .ok_or_else(|| assets_err!("Missing texture name") )?;

        let id = self.textures.len() as u32;
        self.textures.insert(name, Texture { id });

        Ok(())
    }

    fn load_csv(&mut self, init: &GameClientInit, args: &[&str]) -> Result<(), Error> {
        let &csv_name = args.get(1)
            .ok_or_else(|| assets_err!("Missing csv name") )?;

        let csv_string = init.text_assets.get(csv_name)
            .ok_or_else(|| assets_err!("Failed to match csv name to csv data") )?;

        // Each CSV had its own loading procedure
        match csv_name {
            "atlas_sprites" => self.atlas.load_from_csv(csv_string),
            "terrain_sprites" => self.terrain.load_from_csv(csv_string),
            name => {
                warn!("Unknown csv: {:?}", name);
            }
        }

        Ok(())
    }

    fn import_assets_index(&mut self, init: &GameClientInit) -> Result<(), Error> {
        let mut error: Option<Error> = None;

        // Assets index
        crate::shared::split_csv::<5, _>(&init.assets_bundle, |args| {
            let result = match args[0] {
                "TEXTURE" => {
                    self.load_texture(args)
                },
                "CSV" => {
                    self.load_csv(init, args)
                },
                "SHADER" => Ok(()),
                _ => { Err(assets_err!("Unknown asset type {:?}", args[0])) }
            };
    
            if let Err(new_error) = result {
                crate::shared::merge_error(&mut error, new_error)
            }
        });
    
        if let Some(err) = error {
            return Err(err);
        }
    
        Ok(())
    }

}

impl StoreLoad for Assets {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write_string_hashmap(&self.textures);
        writer.write(&self.atlas);
        self.terrain.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut data = Assets::default();
        data.textures = reader.read_string_hashmap();
        data.atlas = reader.try_read()?;
        data.terrain = TerrainSprites::load(reader)?;
        Ok(data)
    }
}

impl StoreLoad for TerrainSprites {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.tile_size);
        writer.write(&self.tileset_width);
        writer.write(&self.tileset_height);
        writer.write(&self.water);
        writer.write(&self.grass);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut terrain = TerrainSprites::default();
        terrain.tile_size = reader.try_read()?;
        terrain.tileset_width = reader.try_read()?;
        terrain.tileset_height = reader.try_read()?;
        terrain.water = reader.try_read()?;
        terrain.grass = reader.try_read()?;
        Ok(terrain)
    }
}

impl Default for Texture {
    fn default() -> Self {
        Texture { id: 0 }
    }
}

impl Default for Assets {
    fn default() -> Self {
        Assets {
            textures: FnvHashMap::default(),
            atlas: AtlasData::default(),
            terrain: TerrainSprites::default(),
        }
    }
}
