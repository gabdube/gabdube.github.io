use zerocopy_derive::{Immutable, IntoBytes, FromBytes};
use crate::error::Error;
use crate::shared::{size, PositionF32, SizeF32, AABB};
use super::Texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, FromBytes, IntoBytes, Immutable)]
pub struct AtlasInfo {
    pub size: f32,
    pub width: f32,
    pub height: f32,
    pub line_height: f32,
    pub ascender: f32,
    pub descender: f32,
    pub glyph_count: u32,
    pub glyph_max: u32,
}

#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
pub struct ComputedGlyph {
    pub position: AABB,
    pub texcoord: AABB,
}

#[derive(Default)]
pub struct TextMetrics {
    pub texture: Texture,
    pub size: SizeF32,
    pub glyphs: Box<[ComputedGlyph]>,
}

impl TextMetrics {
    pub fn point_to_caret_position(&self, point: PositionF32) -> u32 {
        let x = point.x;
        if x <= 0.0 {
            0
        } else if x >= self.size.width {
            self.glyphs.len() as u32
        } else {
            let mut position = 0;
            for (i, g) in self.glyphs.iter().enumerate() {
                let halfpoint = g.position.left + (g.position.size().width / 2.0);
                if x >= halfpoint {
                    position = 1 + i as u32;
                }
            }

            position
        }
    }
}
 
#[repr(C)]
#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
pub struct AtlasGlyph {
    pub unicode: u32,
    pub advance: f32,
    pub atlas_bound: [f32; 4],
    pub plane_bound: [f32; 4],
}

#[derive(Default, Clone)]
pub struct Font {
    pub info: AtlasInfo,
    pub texture: Texture,
    pub glyphs: Vec<AtlasGlyph>,
    pub max_glyph_height: f32,
}

impl Font {

    pub fn from_msdf(texture: Texture, atlas_data: &[u8]) -> Result<Self, Error> {
        let (x, _, y) = unsafe { atlas_data.align_to::<u32>() };
        if x.len() != 0 || y.len() != 0 {
            return Err(assets_err!("Failed to parse font atlas data. Data must be aligned to 4 bytes"));
        }

        let info = unsafe { *(atlas_data.as_ptr() as *const AtlasInfo) };
        let glyph_ptr = unsafe { atlas_data.as_ptr().add(size_of::<AtlasInfo>()) as *const AtlasGlyph };
        let mut glyphs = vec![Default::default(); info.glyph_max as usize];
        let mut max_glyph_height = 0.0;
        
        for i in 0..(info.glyph_count as usize) {
            let glyph: AtlasGlyph = unsafe { glyph_ptr.add(i).read() };
            glyphs[glyph.unicode as usize] = glyph;
            max_glyph_height = f32::max(glyph.plane_bound[1], max_glyph_height);
        }

        let font = Font {
            info,
            texture,
            glyphs,
            max_glyph_height
        };

        Ok(font)
    }

    /// Compute the bounds of character `c` at scale `scale` into `glyph`. Return the advance of the glyph
    pub fn compute_glyph(&self, c: &str, scale: f32, glyph: &mut ComputedGlyph) -> f32 {
        // Multi characters glyph not supported
        let chr = match c.len() == 1 {
            true => c.chars().next().unwrap_or('?'),
            false => '?'
        };

        let atlas_height = self.info.height;
        let atlas_glyph = self.glyphs.get(chr as usize).copied().unwrap_or_default();
        let advance = atlas_glyph.advance * scale;
        let is_space = ((chr == ' ') as u32) as f32;

        // Space characters are zero-width, so we use the advance as width
        // TODO: patch the width in the plane bound
        glyph.position.left = scale * atlas_glyph.plane_bound[0] + (advance * is_space); 
        glyph.position.top = scale * atlas_glyph.plane_bound[3];
        glyph.position.right = scale * atlas_glyph.plane_bound[2] + (advance * is_space);
        glyph.position.bottom = scale * atlas_glyph.plane_bound[1];

        glyph.texcoord.left = atlas_glyph.atlas_bound[0];
        glyph.texcoord.top = atlas_height - atlas_glyph.atlas_bound[3];
        glyph.texcoord.right = atlas_glyph.atlas_bound[2];
        glyph.texcoord.bottom = atlas_height - atlas_glyph.atlas_bound[1];

        advance
    }

    fn compute_text_metrics_inner(&self, text: &str, scale: f32, line_height: f32) -> TextMetrics {
        use unicode_segmentation::UnicodeSegmentation;
        
        let mut glyphs = Vec::with_capacity(text.len());
        let mut advance = 0.0;
        let mut max_height = line_height;
        let mut glyph = ComputedGlyph::default();
        for g in text.graphemes(true) {
            let a = self.compute_glyph(g, scale, &mut glyph);
            glyph.position.left += advance;
            glyph.position.right += advance;

            advance += a;
            max_height = f32::max(max_height, glyph.position.bottom);

            glyphs.push(glyph);
        }

        // Second pass to align the glyph on the bottom
        // This also flips the y axis
        for glyph in glyphs.iter_mut() {
            glyph.position.top = max_height - glyph.position.top;
            glyph.position.bottom = max_height - glyph.position.bottom;
        }

        let size = match text.len() {
            0 => size(0.0, 0.0),
            _ => size(glyph.position.right, max_height)
        };

        TextMetrics {
            texture: self.texture,
            size,
            glyphs: glyphs.into_boxed_slice()
        }
    }

    pub fn compute_text_metrics(&self, text: &str, scale: f32) -> TextMetrics {
        self.compute_text_metrics_inner(text, scale, 0.0)
    }

    pub fn compute_text_metrics_aligned(&self, text: &str, scale: f32) -> TextMetrics {
        self.compute_text_metrics_inner(text, scale, self.max_glyph_height * scale)
    }

    pub fn line_height(&self, scale: f32) -> f32 {
        self.info.line_height * scale
    }

}

impl crate::store::StoreLoad for Font {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.info);
        writer.write(&self.texture);
        writer.write_array(&self.glyphs);
        writer.write(&self.max_glyph_height);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {

        let font = Font {
            info: reader.try_read()?,
            texture: reader.try_read()?,
            glyphs: reader.read_array().to_vec(),
            max_glyph_height: reader.try_read()?,
        };

        Ok(font)
    }
}

impl crate::store::StoreLoad for TextMetrics {
    fn store(&self, writer: &mut crate::store::StoreWriter) {
        writer.write(&self.texture);
        writer.write(&self.size);
        writer.write_array(&self.glyphs);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, Error> {
        let metrics = TextMetrics {
            texture: reader.try_read()?,
            size: reader.try_read()?,
            glyphs: reader.read_array().to_vec().into_boxed_slice(),
        };

        Ok(metrics)
    }
}
