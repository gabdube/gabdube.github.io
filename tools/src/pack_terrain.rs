/*!
Usage: cargo run --release -p tools -- pack-terrain --input-csv sprites.csv --output-image test.png --output-csv test.csv

cargo run --release -p tools -- pack-terrain --input-csv "tools/tinysword_terrain.csv" --output-image "articles/minimal_retained_rust_gui/assets/terrain.png" --output-csv "articles/minimal_retained_rust_gui/assets/terrain.csv"
*/
use std::collections::HashMap;
use crate::helpers::{SpriteData, LoadSpriteParams, CombinedTilemap, InputTilemapTypes, Tilemap15Pieces, BackgroundTile};
use crate::shared;

struct PackWorldArgs {
    output_image_dst: String,
    output_csv_dst: String,
    input_csv: String,
}


#[derive(Default)]
struct PackTerrainState {
    tilemap_inputs: Vec<InputTilemapTypes>,
    tilemap: CombinedTilemap,

    output_image_dst: String,
    output_csv_dst: String,
}

fn args() -> Option<PackWorldArgs> {
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

    if input_csv.is_empty() {
        eprintln!("Input csv file is empty");
        return None;
    }

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

    Some(PackWorldArgs {
        output_image_dst,
        output_csv_dst,
        input_csv,
    })
}

fn parse_string_with_error<T: ::std::str::FromStr>(
    value: Option<&&str>,
    line_no: u32,
    error_msg: &str,
    errors: &mut Vec<String>,
) -> Option<T> {
    match value.and_then(|value| value.parse::<T>().ok() ) {
        Some(value) => Some(value),
        None => { errors.push(format!("{line_no}: {error_msg}")); None }
    }
}

fn load_sprites(args: PackWorldArgs) -> Option<PackTerrainState> {
    const NO_PADDING: u32 = 0;
    
    let mut image_cache: HashMap<String, shared::PngFile> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    let mut line_number = 0;

    let mut state = PackTerrainState {
        output_image_dst: args.output_image_dst,
        output_csv_dst: args.output_csv_dst,
        tilemap_inputs: Vec::with_capacity(8),
        ..Default::default()
    };

    shared::split_csv::<7, _>(&args.input_csv, |args| {
        if args.len() < 3 {
            errors.push(format!("{line_number}: Malformed arguments, sprite must have at least 6 parameters. Got {args:?}"));
            return;
        }

        let name = args[0].to_string();
        let path = args[1].to_string();
        let image = shared::load_cached(&mut image_cache, path);

        match args[2] {
            "background" => {
                let background_tile = BackgroundTile::new(name, SpriteData::load_from_png(&image, LoadSpriteParams::Auto, NO_PADDING));
                state.tilemap_inputs.push(InputTilemapTypes::Background(background_tile));
            },
            "15pieces" => {
                let v0: Option<u32> = parse_string_with_error(args.get(3), line_number, "Missing tile size parameter", &mut errors);
                let v1: Option<u32> = parse_string_with_error(args.get(4), line_number, "Missing offset left parameter", &mut errors);
                let v2: Option<u32> = parse_string_with_error(args.get(5), line_number, "Missing offset top parameter", &mut errors);
                let [tile_size, offset_left, offset_top] = match [v0, v1, v2] {
                    [Some(v0), Some(v1), Some(v2)] => [v0, v1, v2],
                    _ => { return; }
                };

                let offset_right = offset_left + (tile_size * 4);
                let offset_bottom = offset_top + (tile_size * 4);
                let sprite_params = LoadSpriteParams::crop(offset_left, offset_top, offset_right, offset_bottom);

                let tilemap = Tilemap15Pieces {
                    name,
                    data: SpriteData::load_from_png(&image, sprite_params, NO_PADDING),
                    tile_size,
                };

                state.tilemap_inputs.push(InputTilemapTypes::Tilemap15Pieces(tilemap));
            },
            other => {
                errors.push(format!("{line_number}: Unknown sprite type {other:?}, must be [\"15pieces\", \"background\"]"));
                return;
            }
        };
       

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

fn process_terrain_sprites(state: &mut PackTerrainState) -> bool {
    let mut errors = Vec::new();
    
    for input_tilemap in state.tilemap_inputs.drain(..) {
        if let Err(e) = state.tilemap.add_tilemap(input_tilemap) {
            errors.push(e);
        }
    }

    if errors.is_empty() {
        state.tilemap.process();
        true 
    } else {
        eprintln!("Error while processing combined tilemap:");
        for error in errors {
            eprintln!("{error}");
        }
        false
    }
}

fn write_png(state: &PackTerrainState) {
    if let Err(e) = shared::write_png(&state.output_image_dst, &state.tilemap.output_image_pixels, state.tilemap.output_image_size) {
        eprintln!("Failed to write output image to {:?}: {:?}", state.output_image_dst, e);
    }
}

fn write_csv(state: &PackTerrainState) {
    use std::io::Write;
    use std::fs::File;

    let csv_out = state.tilemap.generate_csv();
    let write_result = File::create(&state.output_csv_dst)
        .and_then(|mut file| file.write(csv_out.as_bytes()) );

    if let Err(e) = write_result {
        eprintln!("Failed to write output csv to {:?}: {:?}", state.output_csv_dst, e);
    }
}


pub fn run() -> Option<()> {
    let args = args()?;
    let mut state = load_sprites(args)?;

    if !process_terrain_sprites(&mut state) {
        return None;
    }

    write_png(&state);
    write_csv(&state);

    println!("Processed terrain atlas");

    Some(())
}