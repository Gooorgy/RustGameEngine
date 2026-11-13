#version 450

struct PointLight {
    vec3 position;
    float radius;
    vec3 color;
    float intensity;
};

// G-buffer samplers
layout(set = 0, binding = 1) uniform sampler2D albedoTexture;
layout(set = 0, binding = 2) uniform sampler2D normalTexture;
layout(set = 0, binding = 3) uniform sampler2D depthTexture;

// Shadow pass depths
// TODO: Replace with single uniform
layout(set = 0, binding = 4) uniform sampler2DShadow shadowMapCascade0;
layout(set = 0, binding = 5) uniform sampler2DShadow shadowMapCascade1;
layout(set = 0, binding = 6) uniform sampler2DShadow shadowMapCascade2;
layout(set = 0, binding = 9) uniform sampler2DShadow shadowMapCascade3;
layout(std430, set = 0, binding = 10) readonly buffer pointLights{
    PointLight lights[];
};


// Directionallight cascade view projections
layout(std140, set = 0, binding = 7) uniform Cascade {
    mat4[4] cascadeViewProjMat;
} cascade;

// Camera view + proj
layout(binding = 8) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

// Lighting uniform (ambient and directional light)
layout(std140, set = 0, binding = 0) uniform Lighting {
    vec4 lightDirection;
    vec4 lightColor;
    vec4 ambiantLight;
    vec4 cascadeDepths;
    // PointLight count
    uint lightCount;
} lighting;

layout(location = 0) in vec2 fragTexCoord;
layout(location = 0) out vec4 fragColor;

vec3 reconstructWorldPosition(vec2 fragTexCoord, float depth) {
    vec4 ndcPos = vec4(fragTexCoord * 2.0 - vec2(1.0), depth, 1.0);
    vec4 viewSpacePos = inverse(ubo.proj) * ndcPos;
    viewSpacePos /= viewSpacePos.w;
    vec4 worldPos = inverse(ubo.view) * viewSpacePos;
    return worldPos.xyz;
}

float pcfSample(sampler2DShadow shadowMap, vec2 uv, float compareZ) {
    float shadow = 0.0;
    shadow += textureOffset(shadowMap, vec3(uv, compareZ), ivec2(-1, -1));
    shadow += textureOffset(shadowMap, vec3(uv, compareZ), ivec2( 0, -1));
    shadow += textureOffset(shadowMap, vec3(uv, compareZ), ivec2( 1, -1));
    shadow += textureOffset(shadowMap, vec3(uv, compareZ), ivec2(-1,  0));
    shadow += textureOffset(shadowMap, vec3(uv, compareZ), ivec2( 0,  0));
    shadow += textureOffset(shadowMap, vec3(uv, compareZ), ivec2( 1,  0));
    shadow += textureOffset(shadowMap, vec3(uv, compareZ), ivec2(-1,  1));
    shadow += textureOffset(shadowMap, vec3(uv, compareZ), ivec2( 0,  1));
    shadow += textureOffset(shadowMap, vec3(uv, compareZ), ivec2( 1,  1));
    return shadow / 9.0;
}

float calculateShadow(int cascadeIndex, vec3 worldPos, vec3 normal) {
    mat4 lightViewProj = cascade.cascadeViewProjMat[cascadeIndex];

    // Normal offset bias
    vec3 lightDir = normalize(lighting.lightDirection.xyz);
    float cosTheta = max(dot(normal, lightDir), 0.0);
    float sinTheta = sqrt(1.0 - cosTheta * cosTheta);
    float depthScale = abs(lightViewProj[2][2]);
    float cascadeRes = (cascadeIndex < 2) ? 2048.0 : 1024.0;
    float texelWorldSize = 1.0 / (cascadeRes * depthScale);
    vec3 offsetPos = worldPos + normal * texelWorldSize * max(sinTheta, 0.1) * 2.0;

    // Convert offset position to light space
    vec4 lightSpacePos = lightViewProj * vec4(offsetPos, 1.0);
    lightSpacePos /= lightSpacePos.w;

    // UV coordinates in shadow map
    vec2 uv = lightSpacePos.xy * 0.5 + 0.5;

    // Discard everything outside the frustum
    if (lightSpacePos.z > 1.0 || lightSpacePos.z < 0.0 || uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 0.0;
    }

    float bias = 0.002 * depthScale;

    float z = lightSpacePos.z - bias;
    float shadow;
    if (cascadeIndex == 0) {
        shadow = pcfSample(shadowMapCascade0, uv, z);
    } else if (cascadeIndex == 1) {
        shadow = pcfSample(shadowMapCascade1, uv, z);
    } else if (cascadeIndex == 2) {
        shadow = pcfSample(shadowMapCascade2, uv, z);
    } else {
        shadow = pcfSample(shadowMapCascade3, uv, z);
    }

    float shadowFade = smoothstep(0.0, 0.1, cosTheta);
    return mix(1.0, 1.0 - shadow, shadowFade);
}

// Smooth inverse-square falloff that reaches exactly 0 at the radius
float attenuation(float dist, float radius) {
    float d2 = dist * dist;
    float r2 = radius * radius;
    float window = clamp(1.0 - (d2 * d2) / (r2 * r2), 0.0, 1.0);
    return (window * window) / max(d2, 0.0001);
}

void main() {
    vec3 albedo = texture(albedoTexture, fragTexCoord).rgb;
    vec3 normal = texture(normalTexture, fragTexCoord).rgb;
    float depth = texture(depthTexture, fragTexCoord).r;

    if (depth == 1)
        discard;

    vec4 ndcPos = vec4(fragTexCoord * 2.0 - vec2(1.0), depth, 1.0);
    vec4 viewSpacePos = inverse(ubo.proj) * ndcPos;
    viewSpacePos /= viewSpacePos.w;

    vec3 worldPos = reconstructWorldPosition(fragTexCoord, depth);

    vec3 viewPos =  (ubo.view * vec4(worldPos, 1.0)).xyz;
    float viewDepth = viewPos.z;

    int cascadeIndex = 0;
    for (int i = 0; i < 4 - 1; ++i) {
        if (viewDepth < -lighting.cascadeDepths[i]) {
            cascadeIndex = i + 1;
        }
    }

    vec3 lightDir = normalize(lighting.lightDirection.rgb);
    vec3 cascadeColor = vec3(cascadeIndex == 0 ? 1.0 : 0.0, cascadeIndex == 1 ? 1.0 : 0.0, cascadeIndex == 2 ? 1.0 : 0.0);
    cascadeColor = vec3(1);

    float diff = max(dot(normal, lightDir), 0.0);
    vec3 lightColor = lighting.lightColor.xyz * cascadeColor;
    vec3 diffuse = diff * lightColor * lighting.lightColor.w;

    // apply shadow to diffuse
    float shadow = calculateShadow(cascadeIndex, worldPos, normal);
    diffuse = diffuse * (1.0 - shadow);

    for (uint i = 0; i < lighting.lightCount; ++i) {
        PointLight l = lights[i];

        vec3 toLight = l.position - worldPos;
        float dist = length(toLight);
        if (dist >= l.radius) continue;

        vec3 L = toLight / dist;
        float NdotL = max(dot(normal, L), 0.0);

        diffuse += NdotL * l.color * l.intensity * attenuation(dist, l.radius);
    }


    // add ambient to diffuse
    vec3 ambient = lighting.ambiantLight.rgb * lighting.ambiantLight.w;
    vec3 lightingResult = ambient + diffuse;

    // combine with albedo
    vec3 finalColor = albedo * lightingResult;

    fragColor = vec4(finalColor, 1.0);
}