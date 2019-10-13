#version 450

layout(location = 0) in vec2 a_Pos;
layout(location = 1) in vec2 tex_coords;
layout(location = 0) out vec2 tex_coords_out;

layout(set = 0, binding = 0) uniform _ {
    mat4 matrix;
};

void main() {
    tex_coords_out = tex_coords;
    gl_Position = matrix * vec4(a_Pos, 0.0, 1.0);
    // gl_Position = vec4(a_Pos, 0.0, 1.0);
}
