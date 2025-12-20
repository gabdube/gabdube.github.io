mod fonts;
pub use fonts::{Font, TextMetrics};

use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::error::Error;
use crate::shared::{ExternalId, AABB, parse_f32};
use crate::GameClientInit;
use super::sprites::StaticSprite;

#[derive(Default, Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct Texture {
    pub id: ExternalId,
}

#[derive(Default, Copy, Clone, FromBytes, IntoBytes, Immutable)]
pub struct AtlasData {
    pub texture: Texture,
    pub ferris: StaticSprite,
    pub ferris_happy: StaticSprite,
    pub button_default: StaticSprite,
    pub button_hovered: StaticSprite,
    pub button_pressed: StaticSprite,
}

impl AtlasData {
    fn load_from_csv(&mut self, csv: &str) {
        crate::shared::split_csv::<7, _>(csv, |args| {
            let name = args[0];
            //let frame_count = parse_u32(args.get(1));
            let left = parse_f32(args.get(2));
            let top = parse_f32(args.get(3));
            let right = parse_f32(args.get(4));
            let bottom = parse_f32(args.get(5));
            let sprite = StaticSprite { texcoord: AABB { left, top, right, bottom } };

            match name {
                "ferris" => { self.ferris = sprite; }
                "ferris-happy" => { self.ferris_happy = sprite; }
                "button-default" => { self.button_default = sprite; }
                "button-hovered" => { self.button_hovered = sprite; }
                "button-pressed" => { self.button_pressed = sprite; }
                _ => { warn!("Unknown atlas key {:?}", name) }
            }
        });
    }
}

#[derive(Default)]
pub struct Assets {
    pub next_texture_id: u32,
    pub atlas: AtlasData,
    pub roboto: Font,
}

impl Assets {

    pub fn init(&mut self, init: &GameClientInit) -> Result<(), Error> {
        self.import_assets_index(init)?;
        Ok(())
    }

    fn load_texture(&mut self, args: &[&str]) -> Result<(), Error> {
        let &name = args.get(1)
            .ok_or_else(|| assets_err!("Missing texture name") )?;

        // External id is really just the index in a flat array in the engine side
        let texture = Texture { id: ExternalId(self.next_texture_id) };
        self.next_texture_id += 1;

        match name {
            "atlas" => { self.atlas.texture = texture; },
            name => {
                warn!("Unknown texture: {:?}", name);
            }
        }

        Ok(())
    }

    fn load_font(&mut self, init: &GameClientInit, args: &[&str]) -> Result<(), Error> {
        let &name = args.get(1)
            .ok_or_else(|| assets_err!("Missing font name") )?;

        // External id is really just the index in a flat array in the engine side
        let texture = Texture { id: ExternalId(self.next_texture_id) };
        self.next_texture_id += 1;

        let font_atlas_data = init.bin_assets.get(name)
            .ok_or_else(|| assets_err!("No asset named {:?} in binary data", name) )?;

        match name {
            "roboto" => { self.roboto = Font::from_msdf(texture, font_atlas_data)?; },
            name => {
                warn!("Unknown font: {:?}", name);
            }
        }

        Ok(())
    }

    fn load_csv(&mut self, init: &GameClientInit, args: &[&str]) -> Result<(), Error> {
        let &csv_name = args.get(1)
            .ok_or_else(|| assets_err!("Missing csv name") )?;

        let csv_string = init.text_assets.get(csv_name)
            .ok_or_else(|| assets_err!("No asset named {:?} in text data", csv_name) )?;

        // Each CSV had its own loading procedure
        match csv_name {
            "atlas_sprites" => self.atlas.load_from_csv(csv_string),
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
                "MSDF_FONT" => {
                    self.load_font(init, args)
                }
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

impl crate::store::StoreLoad for Assets {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.atlas);
        self.roboto.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut data = Assets::default();
        data.atlas = reader.try_read()?;
        data.roboto = Font::load(reader)?;
        Ok(data)
    }
}

