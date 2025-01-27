#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec3 fragColor;


layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(binding = 1) uniform UboInstance {
    mat4 model;
} uboInstance;



void main() {
    gl_Position =  ubo.proj * ubo.view * vec4(inPosition, 1.0);
    fragColor = inColor;
    // Red color for the lines
}