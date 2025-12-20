/*!
Requires msdf-atlas-gen.exe (https://github.com/Chlumsky/msdf-atlas-gen)

Usage: cargo run --release -p tools -- process-msdf-font --font [font_path] --output-image [image_path] --output-atlas-data [data_path] --msdfgen [/path/to/msdf-atlas-gen]

cargo run --release -p tools -- process-msdf-font --font "tools/unprocessed_assets/roboto.ttf" --output-image "articles/minimal_retained_rust_gui/assets/roboto.png" --output-atlas-data "articles/minimal_retained_rust_gui/assets/roboto.bin" --msdfgen "F:/projects/bin/msdf-atlas-gen.exe"
*/
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;


#[derive(Debug)]
struct ProcessMsdfFontArgs {
    pub font_path: String,
    pub output_image_path: String,
    pub output_atlas_path: String,
    pub temporary_json_path: PathBuf,
    pub msdfgen_path: String,
}

fn args() -> Option<ProcessMsdfFontArgs> {
    let font_path: String = match crate::shared::get_arg("--font") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter --font \"--font font.ttf\"");
            return None;
        }
    };

    let output_image_path: String = match crate::shared::get_arg("--output-image") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter --output-image \"--output-image file.png\"");
            return None;
        }
    };

    let output_atlas_path: String = match crate::shared::get_arg("--output-atlas-data") {
        Some(value) => value,
        None => {
            eprintln!("Missing required parameter --output-atlas-data \"--output-atlas-data file.dat\"");
            return None;
        }
    };

    let msdfgen_path: String = crate::shared::get_arg("--msdfgen").unwrap_or_default();

    let mut temporary_json_path = PathBuf::from(&output_atlas_path);
    temporary_json_path.set_extension("json");

    Some(ProcessMsdfFontArgs {
        font_path,
        output_image_path,
        output_atlas_path,
        temporary_json_path,
        msdfgen_path,
    })
}

fn save_msdf_atlas_rgba(out_path: &str, image_info: &png::OutputInfo, image_data_rgb: &Vec<u8>) -> bool {
    use std::io::BufWriter;
    
    // Remove old file
    if ::std::fs::exists(out_path).unwrap_or(false) {
        if let Err(e) = ::std::fs::remove_file(out_path) {
            println!("Warning: failed to remove old file {:?}: {:?}", out_path, e);
        }
    }

    let file = match ::std::fs::File::create_new(out_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create file {:?}: {:?}", out_path, e);
            return false;
        }
    };

    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, image_info.width, image_info.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
    encoder.set_source_chromaticities(png::SourceChromaticities::new(
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    ));
    let mut writer = match encoder.write_header() {
        Ok(writer) => writer,
        Err(e) => {
            eprintln!("Failed to write png header {:?}", e);
            return false;
        }
    };

    let mut image_data_rgba: Vec<[u8; 4]> = vec![[0, 0, 0, 0]; (image_info.width * image_info.height) as usize];
    for (i, chunk) in image_data_rgb.chunks(3).enumerate() {
        image_data_rgba[i] = [chunk[0], chunk[1], chunk[2], 255];
    }

    let (_, bytes, _) = unsafe { image_data_rgba.align_to::<u8>() };
    if let Err(e) = writer.write_image_data(bytes) {
        eprintln!("Failed to write image data {:?}", e);
        return false;
    }

    true
}

fn generate_msdf_atlas(msdf_gen_path: &str, input_font: &str, output_image: &str, output_json: &PathBuf) -> bool {
    let final_path = match msdf_gen_path.is_empty() {
        false => msdf_gen_path,
        true => {
            println!("Warning. No path defined for msdf-atlas-gen. Assuming the binary is in PATH");
            "msdf-atlas-gen.exe"
        },
    };
    
    let response_result = Command::new(final_path)
        .arg("-font")
        .arg(input_font)
        .arg("-format")
        .arg("png")
        .arg("-json")
        .arg(output_json)
        .arg("-imageout")
        .arg(output_image)
        .arg("-size")
        .arg("35")
        .output();

    let response = match response_result {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Call to msdf-atlas-gen failed: {:?}", e);
            return false;
        }
    };

    if response.status.code() == Some(1) {
        let output = String::from_utf8(response.stderr).unwrap_or(Default::default());
        eprintln!("msdf-atlas-gen returned an error: {:?}", output);
        return false;
    }

    // Need to convert the RBG png into a RGBA to make sure we use an optimized format on all platform
    let src = match ::std::fs::File::open(&output_image) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("failed to open {}: {:?}", output_image, e);
            return false;
        }
    };

    let decoder = png::Decoder::new(src);
    let mut reader = decoder.read_info().unwrap();
    let mut image_data_rgb: Vec<u8> = vec![0; reader.output_buffer_size()];
    let image_info = reader.next_frame(&mut image_data_rgb).unwrap();

    save_msdf_atlas_rgba(&output_image, &image_info, &image_data_rgb)
}

fn compress_atlas_json(json_path: &PathBuf, bin_dst: &str) -> bool {
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
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

    #[repr(C)]
    #[derive(Copy, Clone, Default)]
    pub struct AtlasGlyph {
        pub unicode: u32,
        pub advance: f32,
        pub atlas_bound: [f32; 4],
        pub plane_bound: [f32; 4],
    }

    fn read_u32(v: &serde_json::Value) -> u32 { v.as_u64().map(|v| v as u32 ).unwrap_or(0) }
    fn read_f32(v: &serde_json::Value) -> f32 { v.as_f64().map(|v| v as f32 ).unwrap_or(0.0f32) }
    fn read_rect(v: &serde_json::Value) -> [f32; 4] {
        match v.as_object() {
            Some(obj) => [
                read_f32(&obj["left"]),
                read_f32(&obj["top"]),
                read_f32(&obj["right"]),
                read_f32(&obj["bottom"])
            ],
            None => [0.0; 4]
        }
    }

    let json_source = match ::std::fs::read_to_string(json_path) {
        Ok(json_source) => json_source,
        Err(e) => {
            eprintln!("Failed to read json file generated by msdf-atlas-gen: {:?}", e);
            return false;
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&json_source) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Failed to parse json file generated by msdf-atlas-gen: {:?}", e);
            return false;
        }
    };

    let atlas = &json["atlas"];
    let metrics = &json["metrics"];
    let glyphs = &json["glyphs"].as_array().unwrap();
    let mut glyph_max = 0;

    let total_size_u32 = (size_of::<AtlasInfo>() + (size_of::<AtlasGlyph>() * glyphs.len())) / size_of::<u32>();
    let mut output: Vec<u32> = vec![0; total_size_u32];

    // Glyph
    unsafe {
        let glyph_dst_base = output.as_mut_ptr().add(size_of::<AtlasInfo>() / 4) as *mut AtlasGlyph;
        let mut offset: isize = 0;
        for glyph in glyphs.iter() {
            let unicode = read_u32(&glyph["unicode"]);
            glyph_max = u32::max(glyph_max, unicode);

            *glyph_dst_base.offset(offset) = AtlasGlyph {
                unicode,
                advance: read_f32(&glyph["advance"]),
                atlas_bound: read_rect(&glyph["atlasBounds"]),
                plane_bound: read_rect(&glyph["planeBounds"]),
            };
            offset += 1;
        }
    }

    // Info
    unsafe {
        let info_dst = output.as_mut_ptr() as *mut AtlasInfo;
        *info_dst = AtlasInfo {
            size: read_f32(&atlas["size"]),
            width: read_f32(&atlas["width"]),
            height: read_f32(&atlas["height"]),
            line_height: read_f32(&metrics["lineHeight"]),
            ascender: read_f32(&metrics["ascender"]),
            descender: read_f32(&metrics["descender"]),
            glyph_count: glyphs.len() as u32,
            glyph_max: glyph_max + 1,
        };
    }

    let (_, output_bytes, _) = unsafe { output.align_to::<u8>() };
    let mut file = ::std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(bin_dst)
        .unwrap();

    if let Err(e) = file.write_all(&output_bytes) {
        eprintln!("Failed to write data to {:?}: {:?}", output_bytes, e);
        return false;
    }

    true
}

pub fn run() -> Option<()> {
    let args = args()?;

    println!("Generating msdf atlas for {:?}", args.font_path);

    if !generate_msdf_atlas(&args.msdfgen_path, &args.font_path, &args.output_image_path, &args.temporary_json_path) {
        eprintln!("Failed to generate msdf atlas");
        return None;
    }

    if !compress_atlas_json(&args.temporary_json_path, &args.output_atlas_path) {
        eprintln!("Failed to compress atlas json");
        return None;
    }

    if let Err(e) = ::std::fs::remove_file(&args.temporary_json_path) {
        println!("Warning: failed to remove atlas json file {:?}: {:?}", args.temporary_json_path, e);
    }

    Some(())
}
