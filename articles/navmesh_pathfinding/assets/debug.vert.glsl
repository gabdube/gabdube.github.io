#version 300 es
    
in vec2 in_position;
in vec4 in_color;

uniform vec2 view_position;
uniform vec2 view_size;

out vec4 color;

void main() {
    color = in_color;
    vec2 pos = ((view_position + in_position) / view_size * vec2(2.0)) - vec2(1.0);
    gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
}
