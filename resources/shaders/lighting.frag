#version 450

// G-buffer samplers
layout(set = 0, binding = 1) uniform sampler2D albedoTexture;  // Albedo texture
layout(set = 0, binding = 2) uniform sampler2D normalTexture;  // Normal texture
layout(set = 0, binding = 3) uniform sampler2D depthTexture;   // Depth texture
layout(set = 0, binding = 9) uniform sampler2D posTexture;   // Depth texture



layout(set = 0, binding = 4) uniform sampler2DShadow shadowMapCascade0;
layout(set = 0, binding = 5) uniform sampler2DShadow shadowMapCascade1;
layout(set = 0, binding = 6) uniform sampler2DShadow shadowMapCascade2;

layout(std140, set = 0, binding = 7) uniform Cascade {
    mat4[3] cascadeViewProjMat;
} cascade;



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

const mat4 biasMat = mat4(
0.5, 0.0, 0.0, 0.0,
0.0, 0.5, 0.0, 0.0,
0.0, 0.0, 1.0, 0.0,
0.5, 0.5, 0.0, 1.0
);

vec3 reconstructWorldPosition(vec2 fragTexCoord, float depth) {
    float z = depth * 2.0 - 1.0;
    vec4 ndcPos = vec4(fragTexCoord * 2.0 - vec2(1.0), depth, 1.0);

    vec4 viewSpacePos = inverse(ubo.proj) * ndcPos;

    viewSpacePos /= viewSpacePos.w;

    vec4 worldPos = inverse(ubo.view) * viewSpacePos;

    return worldPos.xyz;
}

float textureProj(vec3 shadowCoord, vec2 offset, int cascadeIndex) {
    return 0;
}

float calculateShadow(int cascadeIndex, vec3 worldPos, vec3 normal) {
    mat4 lightViewProj = cascade.cascadeViewProjMat[cascadeIndex];

    vec3 lightDir = normalize(lighting.lightDirection.rgb);
    float bias = max(0.01 * (1.0 - dot(normal, lightDir)), 0.0005);
    //bias = 0;
    vec3 offsetWorldPos = worldPos + normal * bias;

    vec4 lightSpacePos = lightViewProj * vec4(offsetWorldPos, 1.0);




    lightSpacePos /= lightSpacePos.w;
    vec2 uv = lightSpacePos.xy * 0.5 + 0.5;

    float shadow = 0.0;

    if(lightSpacePos.z > 1) {
        return shadow;
    }

    float dynamicBias = max(0.005 * tan(acos(dot(normal, lightDir))), 0.001);
    int sampleRadius = 1;
    ivec2 texDim = ivec2(4096,4096);
    vec2 pixelSize = 1.0 / texDim;


    for(int y = -sampleRadius; y <= sampleRadius; y++)
    {
        for(int x = -sampleRadius; x <= sampleRadius; x++)
        {
            shadow += texture(shadowMapCascade1, vec3(uv + vec2(x, y) * pixelSize,lightSpacePos.z));
        }
    }

    shadow /= 9;//pow((sampleRadius * 2 + 1), 2);

    return 1-shadow;
}

float relinearizeDepth(float depth, float zNear, float zFar) {
    return zNear * zFar / (zFar - depth * (zFar - zNear));
}

float sdBox(vec2 p, vec2 b )
{
    vec2 d = abs(p)-b;
    return length(max(d,0.0)) + min(max(d.x,d.y),0.0);
}


void main() {
    // Sample the G-buffer textures
    vec3 albedo = texture(albedoTexture, fragTexCoord).rgb;
    vec3 normal = texture(normalTexture, fragTexCoord).rgb;
    // Compute lighting
    // Sample depth and reconstruct world position
    float depth = texture(depthTexture, fragTexCoord).r;
    vec4 pos = texture(posTexture, fragTexCoord);

    float zView = relinearizeDepth(depth, 0.1, 1000.0);
    zView = clamp(zView, 0.0, 1.0);


    if(depth == 1)
        discard;

    vec4 ndcPos = vec4(fragTexCoord * 2.0 - vec2(1.0), depth, 1.0);

    vec4 viewSpacePos = inverse(ubo.proj) * ndcPos;

    viewSpacePos /= viewSpacePos.w;

    vec3 worldPos = reconstructWorldPosition(fragTexCoord, depth);


    // Determine cascade based on depth

    float shadow = calculateShadow(1, worldPos, normal);

    // Ambient lighting
    vec3 ambient = lighting.ambiantLight.rgb * lighting.ambiantLight.w;

    // Directional lighting
    vec3 lightDir = normalize(lighting.lightDirection.rgb);
    float diff = max(dot(normal, lightDir), 0.0); // Negative for correct direction


    vec3 diffuse = diff * lighting.lightColor.xyz * lighting.lightColor.w;
    diffuse = diffuse * (1.0 - shadow);

    // Combine ambient and diffuse lighting
    vec3 lightingResult = ambient + diffuse;

    // Apply lighting to the albedo color
    vec3 finalColor = albedo * lightingResult;



    fragColor = vec4(finalColor, 1.0);

    //fragColor = vec4(zView,zView,zView, 1.0);
    // Output the final color
    //fragColor = vec4(worldPos, 1.0);
    //fragColor = vec4(uv, 0.0, 1.0);
    //fragColor = vec4(cascadeIndex == 0 ? 1.0 : 0.0, cascadeIndex == 1 ? 1.0 : 0.0, cascadeIndex == 2 ? 1.0 : 0.0, 1.0);
}