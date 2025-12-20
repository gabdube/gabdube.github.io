#version 300 es

in vec2 in_position;
in vec2 in_instance_position;
in vec2 in_instance_texcoord;

uniform vec2 view_position;
uniform vec2 view_size;

out vec2 uv;

void main() {
    const float CELL_SIZE_PX = 64.0;

    uv = vec2(
        in_instance_texcoord.x + (in_position.x*CELL_SIZE_PX),  
        in_instance_texcoord.y + (in_position.y*CELL_SIZE_PX)
    );

    vec2 pos = vec2(
        view_position.x + (in_instance_position.x + (in_position.x * CELL_SIZE_PX)),
        view_position.y + (in_instance_position.y + (in_position.y * CELL_SIZE_PX))
    );

    pos = ((pos / view_size) * 2.0) - 1.0;

    gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
}
