#version 450

// G-buffer samplers
layout(set = 0, binding = 1) uniform sampler2D albedoTexture;  // Albedo texture
layout(set = 0, binding = 2) uniform sampler2D normalTexture;  // Normal texture
layout(set = 0, binding = 3) uniform sampler2D depthTexture;   // Depth texture

// Shadow pass depths
// TODO: Replace with single uniform
layout(set = 0, binding = 4) uniform sampler2DShadow shadowMapCascade0;
layout(set = 0, binding = 5) uniform sampler2DShadow shadowMapCascade1;
layout(set = 0, binding = 6) uniform sampler2DShadow shadowMapCascade2;

// Directionallight cascade view projections
layout(std140, set = 0, binding = 7) uniform Cascade {
    mat4[3] cascadeViewProjMat;
} cascade;

// Camera view + proj
layout(binding = 8) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

// Lighting uniform (ambient and directional light)
layout(std140, set = 0, binding = 0) uniform Lighting {
    vec4 lightDirection;  // Direction of the directional light
    vec4 lightColor;      // Color of the directional light
    vec4 ambiantLight; // Offset 68 (padding to 80 bytes)
    vec4 cascadeDepths;
} lighting;

layout(location = 0) in vec2 fragTexCoord;  // Texture coordinates
layout(location = 0) out vec4 fragColor;  // Final fragment color


vec3 reconstructWorldPosition(vec2 fragTexCoord, float depth) {
    vec4 ndcPos = vec4(fragTexCoord * 2.0 - vec2(1.0), depth, 1.0);
    vec4 viewSpacePos = inverse(ubo.proj) * ndcPos;
    viewSpacePos /= viewSpacePos.w;
    vec4 worldPos = inverse(ubo.view) * viewSpacePos;
    return worldPos.xyz;
}

float calculateShadow(int cascadeIndex, vec3 worldPos, vec3 normal) {
    mat4 lightViewProj = cascade.cascadeViewProjMat[cascadeIndex];

    // Convert to light space
    vec4 lightSpacePos = lightViewProj * vec4(worldPos, 1.0);
    lightSpacePos /= lightSpacePos.w; // Perspective divide

    // UV coordinates in shadow map
    vec2 uv = lightSpacePos.xy * 0.5 + 0.5;

    // Discard everythin outside the frustum prevent oversampling
    if (lightSpacePos.z > 1.0 || uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 0.0; // Not in shadow
    }

    // Align shadow map texels to the voxel grid
    float texelSize = 1.0 / 4096.0;
    uv = floor(uv / texelSize) * texelSize + texelSize * 0.5;


    // Sample shadow map with PCF
    float shadow = 0.0;
    int sampleRadius = 1; // Increase for smoother shadows
    if(cascadeIndex == 0) {
        sampleRadius = 8;
    }
    vec2 pixelSize = vec2(texelSize);

    for (int y = -sampleRadius; y <= sampleRadius; y++) {
        for (int x = -sampleRadius; x <= sampleRadius; x++) {
            vec2 offset = vec2(x, y) * pixelSize;
            if(cascadeIndex == 0) {
                shadow += texture(shadowMapCascade0, vec3(uv + offset, lightSpacePos.z - 0.0046));
            } else if (cascadeIndex == 1) {
                shadow += texture(shadowMapCascade1, vec3(uv + offset, lightSpacePos.z - 0.0046));
            } else {
                shadow += texture(shadowMapCascade2, vec3(uv + offset, lightSpacePos.z - 0.0046));
            }
        }
    }

    // Average PCF samples
    shadow /= pow((sampleRadius * 2 + 1), 2);
    //return 0;
    return 1.0 - shadow; // Return light contribution
}

void main() {
    // Sample the G-buffer textures
    vec3 albedo = texture(albedoTexture, fragTexCoord).rgb;
    vec3 normal = texture(normalTexture, fragTexCoord).rgb;
    float depth = texture(depthTexture, fragTexCoord).r;

    if(depth == 1)
        discard;

    vec4 ndcPos = vec4(fragTexCoord * 2.0 - vec2(1.0), depth, 1.0);
    vec4 viewSpacePos = inverse(ubo.proj) * ndcPos;
    viewSpacePos /= viewSpacePos.w;

    vec3 worldPos = reconstructWorldPosition(fragTexCoord, depth);

    vec3 viewPos =  (ubo.view * vec4(worldPos, 1.0)).xyz;
    float viewDepth = viewPos.z;

    int cascadeIndex = 0;
    for(int i = 0; i < 3 - 1; ++i) {
        if(viewDepth < -lighting.cascadeDepths[i]) {
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

    // add ambient to diffuse
    vec3 ambient = lighting.ambiantLight.rgb * lighting.ambiantLight.w;
    vec3 lightingResult = ambient + diffuse;

    // combine with albedo
    vec3 finalColor = albedo * lightingResult;

    fragColor = vec4(finalColor, 1.0);
}