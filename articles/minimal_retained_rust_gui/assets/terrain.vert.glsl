#version 300 es

in vec2 in_position;
in uint in_instance_data; // [u8; 4] => [position_x, position_y, uv_x, uv_y]

uniform vec2 view_position;
uniform vec2 view_size;
uniform vec2 sprite_position_offset;

out vec2 uv;

void main() {
    const float CELL_SIZE_PX = 64.0;

    float x = float((in_instance_data >> 24u) & 0xFFu) * CELL_SIZE_PX;
    float y = float((in_instance_data >> 16u) & 0xFFu) * CELL_SIZE_PX;
    float uv_x = float((in_instance_data >> 8u) & 0xFFu) * CELL_SIZE_PX;
    float uv_y = float(in_instance_data & 0xFFu) * CELL_SIZE_PX;

    float instance_right = in_position.x*CELL_SIZE_PX;
    float instance_bottom = in_position.y*CELL_SIZE_PX;

    uv = vec2(uv_x + instance_right, uv_y + instance_bottom);

    vec2 pos = vec2(
        view_position.x + sprite_position_offset.x + (x + instance_right),
        view_position.y + sprite_position_offset.y + (y + instance_bottom)
    );

    pos = ((pos / view_size) * 2.0) - 1.0;
    gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
}
