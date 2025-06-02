#version 300 es

in vec2 in_positions;
in vec2 in_texcoord;
in vec4 in_color;

out vec2 uv;
out vec4 color;

uniform vec2 view_size;

void main() {
    gl_Position = vec4(
                      2.0 * in_positions.x / view_size.x - 1.0,
                      1.0 - 2.0 * in_positions.y / view_size.y,
                      0.0,
                      1.0);
    
    uv = in_texcoord;
    color = in_color;
}
