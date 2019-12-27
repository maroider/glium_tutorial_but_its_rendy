#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 1) uniform texture2D colormap;
layout(set = 0, binding = 2) uniform sampler colorsampler;

void main() {
    color = texture(sampler2D(colormap, colorsampler), tex_coords);
}
