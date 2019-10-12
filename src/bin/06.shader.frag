#version 450

layout(location = 0) in vec2 my_attr;
layout(location = 0) out vec4 color;

void main() {
    color = vec4(my_attr, 0.0, 1.0);
}
