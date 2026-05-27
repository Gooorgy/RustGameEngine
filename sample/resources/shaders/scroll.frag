#version 460

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec4 outNormal;
layout(location = 2) out vec4 outOrm;

layout(location = 1) in vec2 fragTexCoord;
layout(location = 3) in vec3 inNormal;

layout(set = 1, binding = 0) uniform sampler2D colorTexture;

void main() {
    vec4 tex = texture(colorTexture, fragTexCoord);
    // cyan tint + UV-driven brightness stripe to distinguish from default PBR
    float stripe = 0.75 + 0.25 * sin(fragTexCoord.x * 20.0);
    outColor  = vec4(tex.r * stripe * 0.4, tex.g * stripe * 0.9, tex.b * stripe * 1.4, tex.a);
    outNormal = vec4(normalize(inNormal), 0.0);
    outOrm    = vec4(0.8, 0.0, 1.0, 0.0);
}
