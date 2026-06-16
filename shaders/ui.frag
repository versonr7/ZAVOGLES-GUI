#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec4 v_color;

layout(location = 0) out vec4 o_color;

layout(set = 0, binding = 0) uniform sampler2D u_atlas;

void main() {
    vec4 tex = texture(u_atlas, v_uv);
    o_color = tex * v_color;
}
