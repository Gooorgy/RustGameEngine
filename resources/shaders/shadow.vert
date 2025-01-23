#version 450
#extension GL_ARB_separate_shader_objects : enable

// todo: pass via specialization constant
#define SHADOW_MAP_CASCADE_COUNT 4

layout(push_constant) uniform PushConsts {
    vec4 position;
    uint cascadeIndex;
} pushConsts;

layout (set = 0, binding = 0) uniform UBO {
    mat4[SHADOW_MAP_CASCADE_COUNT] cascadeViewProjMat;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec3 inNormal;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;
out gl_PerVertex {
    vec4 gl_Position;
};


void main()
{
    gl_Position =  ubo.cascadeViewProjMat[pushConsts.cascadeIndex] * vec4(inPosition, 1);
    fragColor = inColor;
    fragTexCoord = inTexCoord;
}