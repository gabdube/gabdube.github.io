#version 300 es

precision highp float;

in vec2 uv;
in vec4 color;

out vec4 outColor;

uniform sampler2D gui_sampler;

void main() {
    vec4 texture_in_gamma = texture(gui_sampler, uv);
    vec4 frag_color_gamma = color * texture_in_gamma;
    outColor = frag_color_gamma;
}
