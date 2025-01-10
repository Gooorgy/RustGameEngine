#version 450

layout (location = 0) in vec3 inPos;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
    mat4 ff;
} ubo;

layout(binding = 2) uniform UboInstance {
    mat4 model;
} uboInstance;

layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;

void main()
{
    gl_Position = ubo.proj * ubo.view * uboInstance.model * vec4(inPos, 1.0);
}