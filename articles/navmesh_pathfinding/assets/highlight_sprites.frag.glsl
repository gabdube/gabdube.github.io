#version 300 es

precision highp float;

in vec4 highlight_color;
flat in vec4 texcoord_bounds;
in vec2 uv;

out vec4 outColor;

uniform sampler2D sprite_sampler;

void main() {
    vec2 tex_size = vec2(textureSize(sprite_sampler, 0));
    vec2 tex_coord = uv / tex_size;
    vec4 color = texture(sprite_sampler, tex_coord);

    // Outline options
    // Because textures are in an atlas, coordinates must be clamped in the sprite texture itself
    // or else we're going sample the adjacent sprites
    vec2 pixel = vec2(2.5) / tex_size;
    vec4 bounds = texcoord_bounds / vec4(tex_size, tex_size);
    float outline = 0.0;
    outline += ceil(texture(sprite_sampler, vec2(min(tex_coord.x + pixel.x, bounds.z), tex_coord.y)).a - 0.99);
    outline += ceil(texture(sprite_sampler, vec2(max(tex_coord.x - pixel.x, bounds.x), tex_coord.y)).a - 0.99);
    outline += ceil(texture(sprite_sampler, vec2(tex_coord.x, min(tex_coord.y + pixel.y, bounds.w))).a - 0.99);
    outline += ceil(texture(sprite_sampler, vec2(tex_coord.x, max(tex_coord.y - pixel.y, bounds.y))).a - 0.99);

    float outline_mask = outline * (1.0 - ceil(color.a - 0.99)); // outline mask does not include sprite

    outColor.rgb = highlight_color.rgb * outline_mask;
    outColor.a = outline_mask;
}
