#version 300 es

precision highp float;

in vec2 uv;

out vec4 outColor;

uniform sampler2D sprite_sampler;

void main() {
    vec2 tex_size = vec2(textureSize(sprite_sampler, 0));
    vec2 tex_coord = uv / tex_size;
    vec4 color = texture(sprite_sampler, tex_coord);
    color.rgb = color.rgb * color.a;
    outColor = vec4(color.rgb * 0.75, color.a * 0.75);
}
