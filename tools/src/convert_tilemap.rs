/*!
    NOTE: this tool can't perfectly convert tilemaps. You are expected to manually fix any mistakes
    Usage cargo run --release -p tools -- convert-tilemap --input image.png --input-fmt reduced --output image.png --output-fmt dual [--tile-size 64] [--offset-x 0] [--offset-y 0]

cargo run --release -p tools -- convert-tilemap --input "tools/unprocessed_assets/tiny_swords/Terrain/Tilemap_color2.png" --input-fmt reduced --output "tools/processed_assets/tilemap_grass.png" --output-fmt dual --tile-size 64 --offset-x 64 --offset-y 64
*/

use crate::{
    helpers::{LoadSpriteParams, SpriteData},
    shared::{load_png, rect_u32, size_u32, write_png}
};

#[derive(Debug, PartialEq, Copy, Clone)]
enum TilemapType {
    Reduced,
    Dual
}

impl TilemapType {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "reduced" => Some(TilemapType::Reduced),
            "dual" => Some(TilemapType::Dual),
            _ => None,
        }
    }

    fn all() -> &'static [&'static str] {
        &["reduced", "dual"]
    }
}


#[derive(Debug)]
struct ConverTilemapArgs {
    input_image_path: String,
    input_image_format: TilemapType,
    output_image_path: String,
    output_image_format: TilemapType,
    input_offset_x: u32,
    input_offset_y: u32,
    tile_size: u32,
}

struct ConvertTilemapState {
    input_tilemap: SpriteData,
    input_image_format: TilemapType,
    output_tilemap: SpriteData,
    output_image_format: TilemapType,
    output_image_path: String,
    tile_size: u32,
}

fn args() -> Option<ConverTilemapArgs> {
    let input_image_path: String = match crate::shared::get_arg("--input") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter input");
            return None;
        }
    };

    let output_image_path: String = match crate::shared::get_arg("--output") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter output");
            return None;
        }
    };

    let input_image_format_str: String = match crate::shared::get_arg("--input-fmt") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter input-fmt");
            return None;
        }
    };

    let output_image_format_str: String = match crate::shared::get_arg("--output-fmt") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter output-fmt");
            return None;
        }
    };

    let input_image_format = match TilemapType::from_str(&input_image_format_str) {
        Some(value) => value,
        None => {
            eprintln!("Unknown tilemap type {:?}. Must be one of {:?}", input_image_format_str, TilemapType::all());
            return None;
        }
    };

    let output_image_format = match TilemapType::from_str(&output_image_format_str) {
        Some(value) => value,
        None => {
            eprintln!("Unknown tilemap type {:?}. Must be one of {:?}", input_image_format_str, TilemapType::all());
            return None;
        }
    };

    let input_offset_x = crate::shared::get_arg("--offset-x")
        .and_then(|value| { value.parse::<u32>().ok() } )
        .unwrap_or(0);

    let input_offset_y = crate::shared::get_arg("--offset-y")
        .and_then(|value| { value.parse::<u32>().ok() } )
        .unwrap_or(0);

    let tile_size = crate::shared::get_arg("--tile-size")
        .and_then(|value| { value.parse::<u32>().ok() } )
        .unwrap_or(64);

    Some(ConverTilemapArgs {
        input_image_path,
        input_image_format,
        output_image_path,
        output_image_format,
        input_offset_x,
        input_offset_y,
        tile_size,
    })
}

fn validate_args(args: &ConverTilemapArgs) -> bool {
    if args.input_image_format == TilemapType::Reduced && args.output_image_format == TilemapType::Dual {
        return true;
    }

    eprintln!("Input type {:?} and output type {:?} is not supported", args.input_image_format, args.output_image_format);

    false
}

fn load_sprites(args: ConverTilemapArgs) -> Option<ConvertTilemapState> {
    const NO_PADDING: u32 = 0;
    
    let mut state = ConvertTilemapState {
        input_tilemap: SpriteData::default(),
        input_image_format: args.input_image_format,
        output_tilemap: SpriteData::default(),
        output_image_format: args.output_image_format,
        output_image_path: args.output_image_path,
        tile_size: args.tile_size,
    };

    match state.input_image_format {
        TilemapType::Reduced => {
            let image = load_png(&args.input_image_path);

            let offset_left = args.input_offset_x;
            let offset_top = args.input_offset_y;
            let offset_right = offset_left + (state.tile_size * 4);
            let offset_bottom = offset_top + (state.tile_size * 5);
            let sprite_params = LoadSpriteParams::crop(offset_left, offset_top, offset_right, offset_bottom);
            state.input_tilemap = SpriteData::load_from_png(&image, sprite_params, NO_PADDING);
        },
        TilemapType::Dual => unimplemented!()
    }

    match state.output_image_format {
        TilemapType::Dual => {
            let width = state.tile_size * 4;
            let height = state.tile_size * 4;
            state.output_tilemap = SpriteData::empty_from_size(size_u32(width, height));
        },
        TilemapType::Reduced => unimplemented!()
    }


    Some(state)
}

fn reduced_to_dual(state: &mut ConvertTilemapState) {
    let tile_size = state.tile_size as f32;
    let input = &state.input_tilemap;
    let output = &mut state.output_tilemap;

    let make_tile = |x: f32, y: f32, width: f32, height: f32| {
        let left = tile_size*x;
        let top = tile_size*y;
        let right = left + (width * tile_size);
        let bottom = top + (height * tile_size);
        rect_u32(left as u32, top as u32, right as u32, bottom as u32)
    };

    // Filled
    input.copy_pixels(output, make_tile(1.0, 1.0, 1.0, 1.0), make_tile(2.0, 1.0, 1.0, 1.0));

    // Right
    input.copy_pixels(output, make_tile(0.0, 1.0, 0.75, 1.0), make_tile(1.25, 0.0, 0.75, 1.0));

    // Left
    input.copy_pixels(output, make_tile(2.25, 1.0, 0.75, 1.0), make_tile(3.0, 2.0, 0.75, 1.0));

    // Top
    input.copy_pixels(output, make_tile(1.0, 2.25, 1.0, 0.75), make_tile(1.0, 2.0, 1.0, 0.75));

    // Bottom
    input.copy_pixels(output, make_tile(1.0, 0.0, 1.0, 0.75), make_tile(3.0, 0.25, 1.0, 0.75));

    // Bottom corner left
    input.copy_pixels(output, make_tile(2.25, 0.0, 0.75, 0.75), make_tile(0.0, 0.25, 0.75, 0.75));

    // Bottom corner right
    input.copy_pixels(output, make_tile(0.0, 0.0, 0.75, 0.75), make_tile(1.25, 3.25, 0.75, 0.75));

    // Top corner left
    input.copy_pixels(output, make_tile(2.25, 2.25, 0.75, 0.75), make_tile(3.0, 3.0, 0.75, 0.75));

    // Top corner right
    input.copy_pixels(output, make_tile(0.0, 2.25, 0.75, 0.75), make_tile(0.25, 2.0, 0.75, 0.75));

    // Empty top-right left
    input.copy_pixels(output, make_tile(1.0, 0.0, 1.0, 0.75), make_tile(1.0, 1.25, 1.0, 0.75));
    output.copy_pixels_self(make_tile(1.25, 0.0, 0.75, 0.5), make_tile(1.25, 1.0, 0.75, 0.5));

    // Empty top-right corner
    input.copy_pixels(output, make_tile(1.0, 0.0, 1.0, 0.75), make_tile(2.0, 0.25, 1.0, 0.75));
    output.copy_pixels_self(make_tile(3.0, 2.0, 0.75, 0.5), make_tile(2.0, 0.0, 0.75, 0.5));

    // Empty bottom-left corner
    input.copy_pixels(output, make_tile(0.0, 1.0, 0.75, 1.0), make_tile(2.25, 2.0, 0.75, 1.0));
    output.copy_pixels_self(make_tile(1.0, 2.0, 0.5, 0.75), make_tile(2.0, 2.0, 0.5, 0.75));

    // Empty bottom-right corner
    output.copy_pixels_self(make_tile(1.0, 2.0, 1.0, 0.75), make_tile(3.0, 1.0, 1.0, 0.75));
    output.copy_pixels_self(make_tile(3.0, 2.5, 0.75, 0.5), make_tile(3.0, 1.5, 0.75, 0.5));

    // Empty top-left & bottom-right corner
    output.copy_pixels_self(make_tile(0.0, 0.0, 1.0, 1.0), make_tile(2.0, 3.0, 1.0, 1.0));
    output.copy_pixels_self(make_tile(0.25, 2.0, 0.75, 0.75), make_tile(2.25, 3.0, 0.75, 0.75));

    // Empty top-left & bottom-right corner
    output.copy_pixels_self(make_tile(1.0, 3.0, 1.0, 1.0), make_tile(0.0, 1.0, 1.0, 1.0));
    output.copy_pixels_self(make_tile(3.0, 3.0, 0.75, 0.75), make_tile(0.0, 1.0, 0.75, 0.75));
}

fn write_output(state: &ConvertTilemapState) {
    if let Err(e) = write_png(&state.output_image_path, &state.output_tilemap.pixels, state.output_tilemap.size) {
        eprintln!("Failed to write output image to {:?}: {:?}", state.output_image_path, e);
    }
}

pub(super) fn run() -> Option<()> {
    let args = args()?;
    validate_args(&args);
    
    let mut state = load_sprites(args)?;

    match (state.input_image_format, state.output_image_format) {
        (TilemapType::Reduced, TilemapType::Dual) => reduced_to_dual(&mut state),
        _ => unimplemented!()
    }

    write_output(&state);

    Some(())
}

