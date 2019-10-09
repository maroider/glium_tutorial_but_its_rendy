#version 450

layout(location = 0) in vec2 a_Pos;

layout(push_constant) uniform Locals {
    float t;
};

void main() {
    vec2 pos = a_Pos;
    pos.x += t;
    gl_Position = vec4(pos, 0.0, 1.0);
}
