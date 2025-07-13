use crate::shared::{PngFile, SizeU32, RectU32, RectI32, rect_u32, rect_i32};

pub const PIXEL_SIZE: usize = 4; // Size of rgba u8

pub enum LoadSpriteParams {
    Auto,
    Crop(RectU32),
    Animation { frame_size: SizeU32 }
}

impl LoadSpriteParams {

    pub fn crop(left: u32, top: u32, right: u32, bottom: u32) -> Self {
        LoadSpriteParams::Crop(RectU32 { left, top, right, bottom })
    }

    pub fn from_crop_args(args: &[&str]) -> Option<LoadSpriteParams> {
        let left = args.get(0).and_then(|&arg| arg.parse::<u32>().ok() );
        let top = args.get(1).and_then(|&arg| arg.parse::<u32>().ok() );
        let right = args.get(2).and_then(|&arg| arg.parse::<u32>().ok() );
        let bottom = args.get(3).and_then(|&arg| arg.parse::<u32>().ok() );
        match [left, top, right, bottom] {
            [Some(left), Some(top), Some(right), Some(bottom)] => Some(Self::Crop(RectU32 { left, top, right, bottom })),
            _ => None
        }
    }

    pub fn from_animation_args(args: &[&str]) -> Option<LoadSpriteParams> {
        let width = args.get(0).and_then(|&arg| arg.parse::<u32>().ok() );
        let height = args.get(1).and_then(|&arg| arg.parse::<u32>().ok() );
        match [width, height] {
            [Some(width), Some(height)] => Some(Self::Animation { frame_size: SizeU32 { width, height } }),
            _ => None
        }
    }

}

/// A 2D sprite data extracted from an image
#[derive(Default, Debug)]
pub struct SpriteData {
    /// Pixel data of the sprite
    pub pixels: Vec<u8>,
    /// Size of the whole sprite
    pub size: SizeU32,
    /// Size of a single frame in an animation. For simple sprites, this is the value of `size`
    pub frame_size: SizeU32,
}

impl SpriteData {

    pub fn load_from_png(png: &PngFile, params: LoadSpriteParams, padding: u32) -> SpriteData {
        let mut sprite = SpriteData::default();

        match params {
            LoadSpriteParams::Auto => {
                let src_rect = rect_u32(0, 0, png.info.width, png.info.height);
                optimize_simple_sprite(png.info.line_size, &src_rect, &png.data, &mut sprite.size, &mut sprite.pixels, padding);
                sprite.frame_size = sprite.size;
            },
            LoadSpriteParams::Crop(src_rect) => {
                optimize_simple_sprite(png.info.line_size, &src_rect, &png.data, &mut sprite.size, &mut sprite.pixels, padding);
                sprite.frame_size = sprite.size;
            },
            LoadSpriteParams::Animation { frame_size } => {
                use crate::helpers::optimize_animation;
                let mut params = optimize_animation::OptimizeAnimationParams {
                    src_line_size: png.info.line_size,
                    src_rect: rect_u32(0, 0, png.info.width, png.info.height),
                    src_frame_size: frame_size,
                    src_bytes: &png.data,
                    optimized_size: &mut sprite.size,
                    optimized_frame_size: &mut sprite.frame_size,
                    dst_bytes: &mut sprite.pixels,
                    padding: padding as i32
                };

                optimize_animation::optimize_animation(&mut params);
            }
        }

        sprite
    }

    pub fn load_from_sprite_data(data: &SpriteData, params: LoadSpriteParams, padding: u32) -> SpriteData {
        let mut sprite = SpriteData::default();
        match params {
            LoadSpriteParams::Auto => {
                sprite.frame_size = data.frame_size;
                sprite.size = data.size;
                sprite.pixels = data.pixels.clone();
            },
            LoadSpriteParams::Crop(src_rect) => {
                let line_size = (data.frame_size.width as usize) * PIXEL_SIZE;
                optimize_simple_sprite(line_size, &src_rect, &data.pixels, &mut sprite.size, &mut sprite.pixels, padding);
                sprite.frame_size = sprite.size;
            },
            LoadSpriteParams::Animation { .. } => {
                unimplemented!();
            }
        }

        sprite
    }

    pub fn sprite_count(&self) -> u32 {
        self.size.width / self.frame_size.width
    }

    pub fn line_size(&self) -> usize {
        self.size.width as usize * PIXEL_SIZE
    }

}

/// Optimize a sprite. Copying the image delimited by `src_rect` in `src_bytes`, into `dst_rect` and `dst_bytes`, removing the extra unused space around the pixels
fn optimize_simple_sprite(
    src_line_size: usize,
    src_rect: &RectU32,
    src_bytes: &[u8],
    dst_size: &mut SizeU32,
    dst_bytes: &mut Vec<u8>,
    padding: u32
) {
    let mut optimized_rect = RectI32::default();
    optimize_sprite_rect(src_line_size, src_rect, src_bytes, &mut optimized_rect);
    optimize_sprite_copy(src_line_size, src_bytes, &optimized_rect, dst_size, dst_bytes, padding);
}

fn optimize_sprite_rect(
    src_line_size: usize,
    src_rect: &RectU32,
    src_bytes: &[u8],
    optimized_rect: &mut RectI32,
) {
    let mut rect = rect_i32(i32::MAX, i32::MAX, i32::MIN, i32::MIN);

    for y in src_rect.top..src_rect.bottom {
        for x in src_rect.left..src_rect.right {
            let [x2, y2] = [x as usize, y as usize];
            let pixel_offset = (y2 * src_line_size) + (x2 * PIXEL_SIZE) + 3;
            let a: u8 = src_bytes[pixel_offset];
            if a != 0 {
                rect.left = i32::min(rect.left, x as i32);
                rect.right = i32::max(rect.right, x as i32);
                rect.top = i32::min(rect.top, y as i32);
                rect.bottom = i32::max(rect.bottom, y as i32);
            }
        }
    }

    rect.left = i32::max(rect.left, 0);
    rect.top = i32::max(rect.top, 0);
    rect.right = i32::min(rect.right, src_rect.right as i32) + 1;
    rect.bottom = i32::min(rect.bottom, src_rect.bottom as i32) + 1;

    *optimized_rect = rect;
}

fn optimize_sprite_copy(
    src_line_size: usize,
    src_bytes: &[u8],
    optimized_rect: &RectI32,
    dst_size: &mut SizeU32,
    dst_bytes: &mut Vec<u8>,
    padding: u32
) {
    let padding = padding as usize;
    let padding2 = padding + padding;
    let width = padding2 + optimized_rect.width() as usize;
    let height = padding2 + optimized_rect.height() as usize;
    *dst_bytes = vec![0; width * height * PIXEL_SIZE];


    let top = optimized_rect.top as usize;
    let bottom = optimized_rect.bottom as usize;
    let left = optimized_rect.left as usize;
    
    let dst_line_size = width * PIXEL_SIZE;
    let copy_line_size = optimized_rect.width() as usize * PIXEL_SIZE;

    let mut dst_column = padding as usize;
    let mut src_column = top;

    while src_column < bottom {
        let src_bytes_start = (src_column * src_line_size) + (left * PIXEL_SIZE);
        let dst_bytes_start = (dst_column * dst_line_size) + (padding * PIXEL_SIZE);

        unsafe {
            ::std::ptr::copy_nonoverlapping(
                src_bytes.as_ptr().add(src_bytes_start),
                dst_bytes.as_mut_ptr().add(dst_bytes_start),
                copy_line_size
            );
        }

        src_column += 1;
        dst_column += 1;
    }

    *dst_size = SizeU32 { width: width as u32, height: height as u32 };
}

