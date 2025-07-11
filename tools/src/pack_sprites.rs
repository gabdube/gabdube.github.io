/*!
Usage: cargo run --release -p tools -- pack_sprites --input-csv sprites.csv --output-image test.png --output-csv test.csv --max-width 512

cargo run --release -p tools -- pack-sprites --input-csv "tools/unprocessed_assets/minimal_retained_rust_gui_atlas.csv" --output-image "articles/minimal_retained_rust_gui/assets/atlas.png" --output-csv "articles/minimal_retained_rust_gui/assets/atlas.csv" --max-width 712
*/

use std::collections::HashMap;
use crate::helpers::{self, LoadSpriteParams, SpriteData, SpritePackingHelper};
use crate::shared;

const PIXEL_SIZE: usize = 4; // rbga u8
const PACKING_PADDING: u32 = 3;  // Padding (in pixels) to add between each the border of the textures and between each sprites

struct PackSpriteArgs {
    output_image_dst: String,
    output_csv_dst: String,
    input_csv: String,
    max_width: u32,
    premultiply_alpha: bool,
}

#[derive(Debug)]
struct InputSprite {
    name: String,
    data: SpriteData,
    output_rect: shared::RectU32,
}

#[derive(Default)]
struct PackSpriteState {
    input_sprites: Vec<InputSprite>,
    max_width: u32,
    premultiply_alpha: bool,

    output_csv_dst: String,
    output_image_dst: String,
    output_image_pixels: Vec<u8>,
    output_image_size: shared::SizeU32
}

fn args() -> Option<PackSpriteArgs> {
    let input_csv_file: String = match crate::shared::get_arg("--input-csv") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter input csv \"--input-csv file.csv\"");
            return None;
        }
    };

    let input_csv = match ::std::fs::read_to_string(&input_csv_file) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to read file {input_csv_file:?}: {e:?}");
            return None;
        }
    };

    let output_image_dst: String = match crate::shared::get_arg("--output-image") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter --output-image \"--output-image file.png\"");
            return None;
        }
    };

    let output_csv_dst: String = match crate::shared::get_arg("--output-csv") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter --output-csv \"--output-csv file.png\"");
            return None;
        }
    };

    let max_width: u32 = crate::shared::get_arg("--max-width")
        .and_then(|value| value.parse::<u32>().ok() )
        .unwrap_or(512);

    let premultiply_alpha: bool = crate::shared::has_arg("--premultiply-alpha");

    Some(PackSpriteArgs {
        output_image_dst,
        output_csv_dst,
        input_csv,
        max_width,
        premultiply_alpha,
    })
}

fn load_cached<'a>(cache: &'a mut HashMap<String, shared::PngFile>, path: String) -> &'a shared::PngFile {
    if !cache.contains_key(&path) {
        cache.insert(path.clone(), shared::load_png(&path));
    }

    cache.get(&path).unwrap()
}

fn load_sprites(args: PackSpriteArgs) -> Option<PackSpriteState> {
    let mut image_cache: HashMap<String, shared::PngFile> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    let mut line_number = 0;

    let mut state = PackSpriteState {
        max_width: args.max_width,
        output_image_dst: args.output_image_dst,
        output_csv_dst: args.output_csv_dst,
        premultiply_alpha: args.premultiply_alpha,
        ..Default::default()
    };

    shared::split_csv::<7, _>(&args.input_csv, |args| {
        if args.len() < 3 {
            errors.push(format!("{line_number}: Malformed arguments, sprite must have at least 3 parameters. Got {args:?}"));
            return;
        }

        let name = args[0].to_string();
        let path = args[1].to_string();
        let image = load_cached(&mut image_cache, path);

        let ty = match args[2] {
            "auto" => LoadSpriteParams::Auto,
            "crop" => match LoadSpriteParams::from_crop_args(&args[3..]) {
                Some(value) => value,
                None => {
                    errors.push(format!("{line_number}: Failed to parse crop arguments. Expected [left, top, right, bottom], got {:?}", &args[3..]));
                    return;
                }
            },
            "animation" => match LoadSpriteParams::from_animation_args(&args[3..]) {
                Some(value) => value,
                None => {
                    errors.push(format!("{line_number}: Failed to parse crop arguments. Expected [frame_width, frame_height], got {:?}", &args[3..]));
                    return;
                }
            }
            other => {
                errors.push(format!("{line_number}: Unknown sprite type {other:?}, must be one of [\"auto\", \"crop\"]"));
                return;
            }
        };

        let data = SpriteData::load_from_png(&image, ty, PACKING_PADDING);

        state.input_sprites.push(InputSprite { 
            name,
            data,
            output_rect: shared::RectU32::default()
        });

        line_number += 1;
    });

    if errors.len() > 0 {
        eprintln!("Error while reading CSV:");
        for error in errors {
            eprintln!("{error}");
        }
        return None;
    }

    Some(state)
}

fn check_min_size(state: &PackSpriteState) -> bool {
    let content_min_width = state.input_sprites.iter().map(|v| v.data.size.width ).max().unwrap_or(0);
    if content_min_width > state.max_width {
        eprintln!("max width ({}) must be at least as large as the longest sprite ({})", state.max_width, content_min_width);
        return false;
    }

    return true;
}

fn generate_pack_sprites(state: &PackSpriteState) -> Vec<helpers::PackSprite> {
    state.input_sprites.iter().enumerate()
        .map(|(index, sprite)| 
            helpers::PackSprite { 
                index: index as u32,
                size: sprite.data.size,
                rect: Default::default()
            }
        )
        .collect()
}

fn allocate_output_image(state: &mut PackSpriteState, pack: &helpers::SpritePackingHelper) {
    let size = pack.size();
    let dst_stride = size.width as usize * PIXEL_SIZE;
    let total_image_size = size.height as usize * dst_stride;

    state.output_image_size = size;
    state.output_image_pixels = vec![0; total_image_size];
}

fn copy_pack_sprites_to_state(state: &mut PackSpriteState, pack: &helpers::SpritePackingHelper) {
    for packed_sprite in pack.sprites() {
        state.input_sprites[packed_sprite.index as usize].output_rect = packed_sprite.rect;
    }
}

fn copy_sprites_to_output_image(state: &mut PackSpriteState) {
    fn copy_sprite(
        dst: &mut [u8], dst_x: usize, dst_y: usize, dst_stride: usize,
        src: &[u8], src_stride: usize, height: usize
    ) {
        for line in 0..height {
            let src_offset = line * src_stride;
            let dst_offset = ((line+dst_y) * dst_stride) + (dst_x * PIXEL_SIZE);
            unsafe {
                ::std::ptr::copy_nonoverlapping(
                    src.as_ptr().add(src_offset),
                    dst.as_mut_ptr().add(dst_offset),
                    src_stride
                );
            }
        }
    }

    fn copy_sprite_premultiply_alpha(
        dst: &mut [u8], dst_x: usize, dst_y: usize, dst_stride: usize,
        src: &[u8], src_stride: usize, height: usize
    ) {
        let pixel_count = src_stride / PIXEL_SIZE;

        for line in 0..height {
            let src_offset = line * src_stride;
            let dst_offset = ((line+dst_y) * dst_stride) + (dst_x * PIXEL_SIZE);
            
            let mut pixel_offset = 0;
            for _ in 0..pixel_count {
                let [mut r, mut g, mut b, a] = unsafe { std::ptr::read(src.as_ptr().add(src_offset + pixel_offset) as *const [u8; 4]) };
                let a_f64 = a as f64 / 255.0;
                r = ((r as f64) * a_f64) as u8;
                g = ((g as f64) * a_f64) as u8;
                b = ((b as f64) * a_f64) as u8;

                unsafe {
                    std::ptr::write(dst.as_mut_ptr().add(dst_offset + pixel_offset) as *mut [u8; 4], [r, g, b, a]);
                }
                
                pixel_offset += PIXEL_SIZE;
            }
        }
    }

    let dst_stride = state.output_image_size.width as usize * PIXEL_SIZE;
    let dst_bytes = &mut state.output_image_pixels;

    for sprite in state.input_sprites.iter() {
        let dst_rect = sprite.output_rect;
        let dst_x = dst_rect.left as usize;
        let dst_y = dst_rect.top as usize;
        let height = sprite.data.size.height as usize;
        let src_stride = sprite.data.line_size();

        if state.premultiply_alpha {
            copy_sprite_premultiply_alpha(
                dst_bytes, dst_x, dst_y, dst_stride,
                &sprite.data.pixels, src_stride, height,
            );
        } else {
            copy_sprite(
                dst_bytes, dst_x, dst_y, dst_stride,
                &sprite.data.pixels, src_stride, height,
            );
        }
    }
}

fn pack_sprites(state: &mut PackSpriteState) -> bool {
    if !check_min_size(state) {
        return false;
    }

    let packed_sprites = generate_pack_sprites(state);
    let packed = SpritePackingHelper::new(state.max_width, packed_sprites);
    
    allocate_output_image(state, &packed);
    copy_pack_sprites_to_state(state, &packed);
    copy_sprites_to_output_image(state);

    true
}

fn write_png(state: &PackSpriteState) {
    if let Err(e) = shared::write_png(&state.output_image_dst, &state.output_image_pixels, state.output_image_size) {
        eprintln!("Failed to write output image to {:?}: {:?}", state.output_image_dst, e);
    }
}

fn write_csv(state: &PackSpriteState) {
    use std::io::Write;
    use std::fs::File;

    let default_buffer_size = 2000;
    let mut csv_out = String::with_capacity(default_buffer_size);

    for sprite in state.input_sprites.iter() {
        let sprite_count = sprite.data.sprite_count();
        let [left, top, right, bottom] = sprite.output_rect.splat();
        csv_out.push_str(&format!("{};{};{};{};{};{};\n", &sprite.name, sprite_count, left, top, right, bottom));
    }

    let write_result = File::create(&state.output_csv_dst)
        .and_then(|mut file| file.write(csv_out.as_bytes()) );

    if let Err(e) = write_result {
        eprintln!("Failed to write output csv to {:?}: {:?}", state.output_csv_dst, e);
    }
}

pub fn run() -> Option<()> {
    let args = args()?;
    let mut state = load_sprites(args)?;
    
    if !pack_sprites(&mut state) {
        return None;
    }

    write_png(&state);
    write_csv(&state);

    println!("Processed {} sprite(s)", state.input_sprites.len());

    Some(())
}
