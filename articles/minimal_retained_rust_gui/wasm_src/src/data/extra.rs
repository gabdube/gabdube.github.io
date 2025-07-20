use crate::shared::pos;

pub(super) fn debug_world_grid(data: &mut super::GameData) {
    debug_grid(data, false);
}

pub(super) fn debug_display_grid(data: &mut super::GameData) {
    debug_grid(data, true);
}

fn debug_grid(data: &mut super::GameData, is_display: bool) {
    let sprite_size = super::terrain::TERRAIN_SPRITE_SIZE;
    let half_sprite_size = sprite_size * 0.5;

    let mut offset_base = 0.0;
    let mut color = [255, 0, 0, 255];
    if is_display {
        offset_base += half_sprite_size;
        color = [0, 255, 0, 255];
    }

    let debug = &mut data.debug;
    let terrain = &data.terrain;
    let terrain_width = (terrain.width() as f32 * sprite_size) - offset_base;
    let terrain_height = (terrain.height() as f32 * sprite_size) - offset_base;

    let mut count = offset_base;
    while count <= terrain_width {
        debug.draw_line(pos(count, offset_base), pos(count, terrain_height), color);
        count += sprite_size;
    }

    count = offset_base;
    while count <= terrain_height {
        debug.draw_line(pos(offset_base, count), pos(terrain_width, count), color);
        count += sprite_size;   
    }
}
