#version 460


#extension GL_ARB_separate_shader_objects: enable

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec4 outNormal;
layout(location = 2) out vec4 outOrm;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 3) in vec3 inNormal;
layout(location = 2) in vec3 inPos;

#ifdef HAS_BASE_COLOR
layout(set = 1, binding = 0) uniform sampler2D baseColor;
#endif

#ifdef HAS_NORMAL
layout(set = 1, binding = 1) uniform sampler2D normal;
#endif

#ifdef HAS_ORM
layout(set = 1, binding = 2) uniform sampler2D orm;
#endif

layout(push_constant) uniform MaterialConstants {
    layout(offset = 16) vec4 baseColor;
    vec4 normal;
    float roughness;
    float metallic;
    float occlusion;
    float speccular;
} pc;



void main() {
    #ifdef HAS_BASE_COLOR
    outColor = texture(baseColor, fragTexCoord);
    #else
    outColor = pc.baseColor;
    #endif

    #ifdef HAS_NORMAL
    outNormal = texture(normal, fragTexCoord);
    #else
    outNormal = vec4(inNormal, 0);
    #endif

    #ifdef HAS_ORM
    outOrm = texture(orm, fragTexCoord);
    #else
    outOrm = vec4(pc.roughness, pc.metallic, pc.occlusion, pc.speccular);
    #endif
}