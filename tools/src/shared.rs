use std::fs::File;

pub struct PngFile {
    pub info: png::OutputInfo,
    pub data: Vec<u8>,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct SizeU32 {
    pub width: u32,
    pub height: u32
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RectU32 {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl RectU32 {
    pub fn splat(&self) -> [u32; 4] {
        [self.left, self.top, self.right, self.bottom]
    } 

    pub fn width(&self) -> u32 {
        self.right - self.left
    }

    pub fn height(&self) -> u32 {
        self.bottom - self.top
    }

    pub fn fits(&self, width: u32, height: u32) -> bool {
        self.width() >= width && self.height() >= height
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RectI32 {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl RectI32 {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

pub fn get_arg(name: &str) -> Option<String> {
    let position = ::std::env::args().position(|arg| arg.as_str() == name);
    position.and_then(|p| ::std::env::args().skip(p+1).next() )
}

pub fn split_csv<const MAX_ARGS: usize, CB: FnMut(&[&str])>(csv: &str, mut callback: CB) {
    let mut start = 0;
    let mut end = 0;
    let mut chars_iter = csv.chars();
    let mut args: [&str; MAX_ARGS] = [""; MAX_ARGS];
    while let Some(c) = chars_iter.next() {
        end += 1;
        if c == '\n' {
            let line = &csv[start..end];
            let mut args_count = 0;
            for substr in line.split(';') {
                if args_count < MAX_ARGS {
                    args[args_count] = substr;
                    args_count += 1;
                }
            }

            if args_count > 1 {
                callback(&args[0..args_count]);
            }

            start = end;
        }
    }
}

pub fn load_png(path: &str) -> PngFile {
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            panic!("Failed to open {path:?}: {e:?}");
        }
    };

    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();

    let mut bytes = vec![0; reader.output_buffer_size()];
    let image_info = reader.next_frame(&mut bytes).unwrap();

    match (image_info.bit_depth, image_info.color_type) {
        (png::BitDepth::Eight, png::ColorType::Rgba) => { /* OK */ },
        combined => unimplemented!("batching sprites for {:?} is not implemented", combined)
    }

    PngFile {
        info: image_info,
        data: bytes
    }
}

pub fn write_png(out_path: &str, pixels_data: &[u8], size: SizeU32) -> Result<(), Box<dyn ::std::error::Error>> {
    use std::io::BufWriter;

    let file = File::create(&out_path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, size.width, size.height);
    encoder.set_compression(png::Compression::Best);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
    let source_chromaticities = png::SourceChromaticities::new(
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header()?;

    writer.write_image_data(pixels_data)?;

    Ok(())
}

pub const fn size_u32(width: u32, height: u32) -> SizeU32 {
    SizeU32 { width, height }
}

pub const fn rect_u32(left: u32, top: u32, right: u32, bottom: u32) -> RectU32 {
    RectU32 { left, top, right, bottom }
}

pub const fn rect_i32(left: i32, top: i32, right: i32, bottom: i32) -> RectI32 {
    RectI32 { left, top, right, bottom }
}
