#version 300 es

precision highp float;

in vec4 highlight_color;
in vec2 uv;

out vec4 outColor;

uniform sampler2D sprite_sampler;

void main() {
    vec2 tex_size = vec2(textureSize(sprite_sampler, 0));
    vec2 tex_coord = uv / tex_size;
    vec4 color = texture(sprite_sampler, tex_coord);

    // Outline options
    // Pixel value cannot be greater than sprite padding in the atlas or else we're going sample the adjacent sprites
    vec2 pixel = vec2(2.0) / tex_size;
    float outline = 0.0;
    outline += ceil(texture(sprite_sampler, vec2(tex_coord.x + pixel.x, tex_coord.y)).a - 0.99);
    outline += ceil(texture(sprite_sampler, vec2(tex_coord.x - pixel.x, tex_coord.y)).a - 0.99);
    outline += ceil(texture(sprite_sampler, vec2(tex_coord.x, tex_coord.y + pixel.y)).a - 0.99);
    outline += ceil(texture(sprite_sampler, vec2(tex_coord.x, tex_coord.y - pixel.y)).a - 0.99);

    float outline_mask = outline * (1.0 - ceil(color.a - 0.99)); // outline mask does not include sprite

    outColor.rgb = highlight_color.rgb * outline_mask;
    outColor.a = outline_mask;
}
