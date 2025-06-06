use fnv::FnvHashMap;
use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::error::Error;
use crate::shared::AABB;
use crate::store::StoreLoad;
use crate::GameClientInit;
use super::base::{AnimatedSprite, StaticSprite};

#[derive(Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct Texture {
    // The unique ID of the texture that identify the resource on the engine side
    pub id: u32,
}

#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
pub struct AtlasData {
    pub texture: Texture,
    pub pawn_idle: AnimatedSprite,
    pub pawn_walk: AnimatedSprite,
    pub castle: StaticSprite,
    pub house: StaticSprite,
}

impl AtlasData {
    pub fn load_csv(&mut self, csv: &str) {
        fn parse(v: &str) -> f32 { str::parse::<f32>(v).unwrap_or(0.0) }
        fn parse_u32(v: &str) -> u32 { str::parse::<u32>(v).unwrap_or(0) }

        crate::shared::split_csv::<6, _>(csv, |args| {
            let name = args[0];
            let frame_count = parse_u32(args[1]);
            let left = parse(args[2]);
            let top = parse(args[3]);
            let right = parse(args[4]);
            let bottom = parse(args[5]);

            match name {
                "pawn_idle" => { self.pawn_idle = AnimatedSprite { sprite_base: AABB { left, top, right, bottom }, frame_count }; }
                "pawn_walk" => { self.pawn_walk = AnimatedSprite { sprite_base: AABB { left, top, right, bottom }, frame_count }; }
                "knight_castle" => { self.castle = StaticSprite { texcoord: AABB { left, top, right, bottom } }; }
                "knight_house" => { self.house = StaticSprite { texcoord: AABB { left, top, right, bottom } }; }
                _ => { warn!("Unknown atlas key {:?}", name) }
            }
        });
    }
}

pub struct Assets {
    pub textures: FnvHashMap<String, Texture>,
    pub fonts: FnvHashMap<String, Vec<u8>>,
    pub atlas: AtlasData
}

impl Assets {

    pub fn init(&mut self, init: &GameClientInit) -> Result<(), Error> {
        self.import_assets_index(init)?;

        self.atlas.texture = self.textures.get("atlas")
            .copied()
            .ok_or_else(|| assets_err!("Missing texture \"atlas\" ") )?;

        self.textures.insert("TEST".to_string(), Texture { id: 999 });

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
            "atlas_sprites" => self.atlas.load_csv(csv_string),
            name => {
                warn!("Unknown csv: {:?}", name);
            }
        }

        Ok(())
    }

    fn load_font(&mut self, init: &GameClientInit, args: &[&str]) -> Result<(), Error> {
        let &font_name = args.get(1)
            .ok_or_else(|| assets_err!("Missing font name") )?;

        let font_data = init.bin_assets.get(font_name)
            .ok_or_else(|| assets_err!("Failed to match font name to font data") )?;

        self.fonts.insert(font_name.to_string(), font_data.clone());

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
                "FONT" => {
                    self.load_font(init, args)
                }
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
        writer.write_string_array_hashmap(&self.fonts);
        writer.write(&self.atlas);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut data = Assets::default();
        data.textures = reader.read_string_hashmap();
        data.fonts = reader.read_string_array_hashmap();
        data.atlas = reader.try_read()?;
        Ok(data)
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
            fonts: FnvHashMap::default(),
            atlas: AtlasData::default(),
        }
    }
}
