#version 450
#define SHADOW_MAP_CASCADE_COUNT 4

layout(set = 0, binding = 0) uniform UBO {
    mat4[SHADOW_MAP_CASCADE_COUNT] cascadeViewProjMat;
} ubo;

layout(std430, set = 0, binding = 1) readonly buffer Transforms {
    mat4 model[];
};

layout(push_constant) uniform PushConsts {
    uint objectIndex;
    uint cascadeIndex;
} pushConsts;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec3 inNormal;

void main() {
    mat4 modelMat = model[pushConsts.objectIndex];
    gl_Position = ubo.cascadeViewProjMat[pushConsts.cascadeIndex] * modelMat * vec4(inPosition, 1);
}
