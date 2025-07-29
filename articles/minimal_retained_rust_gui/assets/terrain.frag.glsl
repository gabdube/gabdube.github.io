#version 300 es

precision highp float;

in vec2 uv;

out vec4 outColor;

uniform sampler2D u_texture;
    
void main() {
    vec2 tex_size = vec2(textureSize(u_texture, 0));
    vec2 tex_coord = uv / tex_size;
    outColor = texture(u_texture, tex_coord);
}
