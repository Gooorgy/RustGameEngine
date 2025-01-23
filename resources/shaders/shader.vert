#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(binding = 1) uniform UboInstance {
    mat4 model;
} uboInstance;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec3 inNormal;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;
layout(location = 2) out vec3 worldPos;
layout(location = 3) out vec3 fragNormal;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    mat3 normalMatrix = transpose(mat3(inverse(uboInstance.model)));
    fragNormal = normalize(normalMatrix * inNormal);

    vec4 worldPosition = uboInstance.model * vec4(inPosition, 1.0);
    worldPos = worldPosition.xyz;
    gl_Position = ubo.proj * ubo.view * vec4(inPosition, 1.0);
    fragColor = inColor;
    fragTexCoord = inTexCoord;
}