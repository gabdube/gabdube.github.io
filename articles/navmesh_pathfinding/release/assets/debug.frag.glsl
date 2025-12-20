#version 300 es

precision highp float;

in vec4 color;

out vec4 outColor;

void main() {
    outColor = vec4(color.rgb * color.a, color.a);
}
