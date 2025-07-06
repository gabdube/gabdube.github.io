use crate::shared::{OffsetU32, RectU32, RectI32, SizeU32, rect_u32, rect_i32, size_u32};
use super::PIXEL_SIZE;

const PADDING: i32 = 2; // Value added when scanning sprites

#[derive(Copy, Clone, Debug)]
struct SpriteScan {
    rect: RectI32,
    offsets: RectI32,
}

#[derive(Copy, Clone, Debug)]
struct SpriteCopy {
    pub src: OffsetU32,
    pub dst: OffsetU32,
}

pub struct OptimizeAnimationParams<'a> {
    pub src_line_size: usize,
    pub src_rect: RectU32,
    pub src_frame_size: SizeU32,
    pub src_bytes: &'a [u8],
    pub optimized_size: &'a mut SizeU32,
    pub optimized_frame_size: &'a mut SizeU32,
    pub dst_bytes: &'a mut Vec<u8>,
}
 
pub fn optimize_animation(params: &mut OptimizeAnimationParams) {
    validations(params);
    let scan = scan_sprites(params);
    let copies = compute_sprite_copies(params, scan);
    generate_final_sprite(params, copies);
}

fn validations(params: &OptimizeAnimationParams) {
    let width = params.src_rect.width();
    if width % params.src_frame_size.width != 0 {
        panic!("Animated sprite with is not a multiple of frame size width");
    }

    let height = params.src_rect.height();
    if height != params.src_frame_size.height {
        panic!("Animated sprite height must be equal to frame size height");
    }
}

fn scan_single_sprite(params: &OptimizeAnimationParams, scan_rect: &RectU32) -> SpriteScan {
    let mut rect = rect_i32(i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    let src_line_size = params.src_line_size;
    let src_bytes = params.src_bytes;

    for y in scan_rect.top..scan_rect.bottom {
        for x in scan_rect.left..scan_rect.right {
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

    rect.left = i32::max(rect.left - PADDING, 0);
    rect.top = i32::max(rect.top - PADDING, 0);
    rect.right = i32::min(rect.right + PADDING, params.src_rect.right as i32);
    rect.bottom = i32::min(rect.bottom + PADDING, params.src_rect.bottom as i32);

    let center_x = (scan_rect.left + (scan_rect.width() / 2)) as i32;
    let center_y = (scan_rect.top + (scan_rect.height() / 2)) as i32;
    let offsets = RectI32 {
        left: center_x - rect.left,
        top: center_y - rect.top,
        right: rect.right - center_x,
        bottom: rect.bottom - center_y,
    };

    SpriteScan {
        rect,
        offsets,
    }
}

/// Scan the sprites in the animation and return their bounding box
fn scan_sprites(params: &OptimizeAnimationParams) -> Vec<SpriteScan> {
    let sprites_count = params.src_rect.width() / params.src_frame_size.width;
    let mut scan = Vec::with_capacity(sprites_count as usize);

    let [left, top, _, bottom] = params.src_rect.splat();
    let frame_width = params.src_frame_size.width;

    for i in 0..sprites_count {
        let scan_rect = rect_u32(left + (i*frame_width), top, left+((i+1)*frame_width), bottom);
        let scan_sprite = scan_single_sprite(params, &scan_rect);
        // println!("{:?}", scan_sprite);
        scan.push(scan_sprite);
    }

    scan
}

/// Compute coordinates to copy a sprite from source to the destination
fn compute_sprite_copies(params: &mut OptimizeAnimationParams, sprite_scan: Vec<SpriteScan>) -> Vec<SpriteCopy> {
    let mut offsets_max = RectI32::default();
    for scan in sprite_scan.iter() {
        offsets_max.left = i32::max(offsets_max.left, scan.offsets.left);
        offsets_max.top = i32::max(offsets_max.top, scan.offsets.top);
        offsets_max.right = i32::max(offsets_max.right, scan.offsets.right);
        offsets_max.bottom = i32::max(offsets_max.bottom, scan.offsets.bottom);
    }
    
    let sprites_count = sprite_scan.len();
    let frame_size = size_u32((offsets_max.left + offsets_max.right) as u32, (offsets_max.top + offsets_max.bottom) as u32);
    let mut sprite_copies = Vec::with_capacity(sprites_count);

    let mut dst_x = 0;

    for scan in sprite_scan {
        let src_x = scan.rect.left - (offsets_max.left - scan.offsets.left);
        let src_y = scan.rect.top - (offsets_max.top - scan.offsets.top);

        let copy = SpriteCopy {
            src: OffsetU32 { x: src_x as u32, y: src_y as u32 },
            dst: OffsetU32 { x: dst_x, y: 0 }
        };

        //println!("{:?}", copy);

        dst_x += frame_size.width;
        sprite_copies.push(copy);
    }
    
    *params.optimized_size = size_u32(frame_size.width * (sprites_count as u32), frame_size.height);
    *params.optimized_frame_size = frame_size;

    // println!("{:?}", params.optimized_size);
    // println!("{:?}", frame_size);

    sprite_copies
}

fn copy_sprite(
    src: &[u8], src_line_size: usize,
    dst: &mut [u8], dst_line_size: usize,
    sprite: &SpriteCopy, width: usize, height: usize
) {
    let src_ptr = src.as_ptr();
    let dst_ptr = dst.as_mut_ptr();

    let src_x = sprite.src.x as usize;
    let src_y = sprite.src.y as usize;

    let dst_x = sprite.dst.x as usize;
    let dst_y = sprite.dst.y as usize;

    let line_size = width * PIXEL_SIZE;

    let max_src_offset = ((src_y + (height-1)) * src_line_size) + ((src_x + width) * PIXEL_SIZE);
    assert!(max_src_offset <= src.len(), "Source read offset is out of the buffer range. {} {}", src_x, src_y);

    let max_dst_offset = ((dst_y + (height-1)) * dst_line_size) + ((dst_x + width) * PIXEL_SIZE);
    assert!(max_dst_offset <= dst.len(), "Dst write offset is out of the buffer range. {} <= {}", max_dst_offset, dst.len());

    for y in 0..height {
        let src_offset = ((src_y + y) * src_line_size) + (src_x * PIXEL_SIZE);
        let dst_offset = ((dst_y + y) * dst_line_size) + (dst_x * PIXEL_SIZE);
        unsafe { ::std::ptr::copy_nonoverlapping(src_ptr.add(src_offset), dst_ptr.add(dst_offset), line_size); }
    }
}

/// Copy the sprites for their source to the final bitmap
fn generate_final_sprite(params: &mut OptimizeAnimationParams, copies: Vec<SpriteCopy>) {
    let width = params.optimized_size.width as usize;
    let height = params.optimized_size.height as usize;
    let dst_line_size = width * PIXEL_SIZE;
    let mut dst_bytes = vec![0; width * height * PIXEL_SIZE];

    let frame_width = params.optimized_frame_size.width as usize;
    let frame_height = params.optimized_frame_size.height as usize;
    for copy in copies.iter() {
        copy_sprite(params.src_bytes, params.src_line_size, &mut dst_bytes, dst_line_size, copy, frame_width, frame_height);
    }

    *params.dst_bytes = dst_bytes;
}
