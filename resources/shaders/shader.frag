#version 460


#extension GL_ARB_separate_shader_objects: enable

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec4 outNormal;
layout(location = 2) out vec4 outPos;


layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 3) in vec3 inNormal;
layout(location = 2) in vec3 inPos;


layout(binding = 1) uniform sampler2D texSampler;
layout(binding = 2) uniform sampler2D normalSampler;
layout(binding = 3) uniform sampler2D funnySampler;

void main() {
    outColor = vec4(1.0, 0.0, 0.0, 1.0);//texture(texSampler, fragTexCoord);
    outNormal = vec4(normalize(inNormal), 1.0);
    outPos = vec4(inPos, 1.0);
    //outColor = vec4(fragColor, 1.0);
}