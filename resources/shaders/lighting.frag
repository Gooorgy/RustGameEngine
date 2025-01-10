#version 450

// G-buffer samplers
layout(set = 0, binding = 1) uniform sampler2D albedoTexture;  // Albedo texture
layout(set = 0, binding = 2) uniform sampler2D normalTexture;  // Normal texture
layout(set = 0, binding = 3) uniform sampler2D depthTexture;   // Depth texture

// Lighting uniform (ambient and directional light)
layout(std140, set = 0, binding = 0) uniform Lighting {
    vec4 lightDirection;  // Direction of the directional light
    vec4 lightColor;      // Color of the directional light
    vec4 ambiantLight; // Offset 68 (padding to 80 bytes)
} lighting;

layout(location = 0) in vec2 fragTexCoord;  // Texture coordinates

layout(location = 0) out vec4 fragColor;  // Final fragment color

void main() {
    // Sample the G-buffer textures
    vec3 albedo = texture(albedoTexture, fragTexCoord).rgb;
    vec3 normal = vec3(0.0, 0.0, 1.0);
    // Compute lighting

    // Ambient lighting
    vec3 ambient = lighting.ambiantLight.rgb * lighting.ambiantLight.w;

    // Directional lighting
    vec3 lightDir = normalize(lighting.lightDirection.rgb);
    float diff = max(dot(normal, -lightDir), 0.0); // Negative for correct direction

    // Diffuse lighting
    vec3 diffuse = diff * lighting.lightColor.rgb * lighting.lightColor.w;

    // Combine ambient and diffuse lighting
    vec3 lightingResult = ambient + diffuse;

    // Apply lighting to the albedo color
    vec3 finalColor = albedo * lightingResult;

    // Output the final color
    fragColor = vec4(finalColor, 1.0);
}