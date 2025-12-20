#version 300 es
    
in vec2 in_position;
in vec2 in_texcoord;
in vec4 in_color;
in uint in_data;

uniform vec2 view_size;

out vec2 texcoord;
out vec4 color;
flat out uint data;

void main() {
    texcoord = in_texcoord;
    color = in_color;
    data = in_data;
    vec2 pos = (in_position / view_size * vec2(2.0)) - vec2(1.0);
    gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
}
