#version 300 es

in vec2 in_positions;
in vec2 in_texcoord;

uniform vec2 view_size;

out vec2 uv;

void main() {
    uv = in_texcoord;
    vec2 pos = (in_positions / view_size * vec2(2.0)) - vec2(1.0);
    gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
}
