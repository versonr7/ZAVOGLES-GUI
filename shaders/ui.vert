#version 450

layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;
layout(location = 2) in vec4 a_color;

layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec4 v_color;

layout(push_constant) uniform PushConstants {
    vec2 u_resolution;
} pc;

void main() {
    vec2 clip = (a_pos / pc.u_resolution) * 2.0 - 1.0;
    clip.y = -clip.y; // Flip Y for Vulkan
    gl_Position = vec4(clip, 0.0, 1.0);
    v_uv = a_uv;
    v_color = a_color;
}
