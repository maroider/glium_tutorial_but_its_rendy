#version 450

layout(location = 0) in vec2 a_Pos;

layout(set = 0, binding = 0) uniform Locals {
    mat4 matrix;
};

void main() {
    gl_Position = matrix * vec4(a_Pos, 0.0, 1.0);
}
