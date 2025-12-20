#version 300 es

precision highp float;

in vec2 texcoord;
in vec4 color;
flat in uint data;

out vec4 outColor;

uniform sampler2D font_texture;
uniform sampler2D image_texture;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    // Image sampling
    vec2 images_texcoord = texcoord / vec2(textureSize(image_texture, 0));
    vec4 color_sample = texture(image_texture, images_texcoord);
    
    // Msdf font sampling
    vec2 fonts_texcoord = texcoord / vec2(textureSize(font_texture, 0));
    vec4 font_sample = texture(font_texture, fonts_texcoord);

    float dist = median(font_sample.r, font_sample.g, font_sample.b);
    float dx = dFdx(texcoord.x);
    float dy = dFdy(texcoord.y);
    float toPixels = 8.0 * inversesqrt(dx * dx + dy * dy);
    float w = fwidth(dist) / 1.5;   
    float font_opacity = smoothstep(0.5 - w, 0.5 + w, dist);

    if (data == 1u) {
        // Flat color
        outColor = vec4(color);
    } else if (data == 2u) {
        // Font
        outColor = vec4(color.rbg, 1.0) * vec4(font_opacity);
    } else {
        // Textured
        outColor = color_sample;
    }
}
