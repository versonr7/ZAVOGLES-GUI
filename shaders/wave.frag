#version 450

layout(location = 0) in vec2 v_uv;

layout(location = 0) out vec4 o_color;

layout(push_constant) uniform PushConstants {
    float u_time;
    vec2 u_resolution;
} pc;

void main() {
    vec2 uv = v_uv;
    
    // XMB-style wave
    float wave1 = sin(uv.x * 10.0 + pc.u_time * 2.0) * 0.05;
    float wave2 = sin(uv.x * 15.0 + pc.u_time * 1.5) * 0.03;
    float wave3 = sin(uv.x * 7.0 + pc.u_time * 2.5) * 0.04;
    
    float intensity = wave1 + wave2 + wave3;
    
    vec3 color1 = vec3(0.0, 0.2, 0.6);  // Deep blue
    vec3 color2 = vec3(0.0, 0.5, 0.9);  // Light blue
    
    vec3 final_color = mix(color1, color2, uv.y + intensity);
    
    // Fade at edges
    float alpha = smoothstep(0.0, 0.3, uv.y) * smoothstep(1.0, 0.7, uv.y);
    alpha *= 0.6; // Semi-transparent
    
    o_color = vec4(final_color, alpha);
}
