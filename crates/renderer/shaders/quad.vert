#version 450

layout(location = 0) out vec2 fragTexCoord; // Pass-through texture coordinates

void main() {
    // Define a fullscreen triangle
    vec2 positions[3] = vec2[](
    vec2(-1.0, -1.0), // Bottom-left
    vec2( 3.0, -1.0), // Bottom-right
    vec2(-1.0,  3.0)  // Top-left
    );

    vec2 texCoords[3] = vec2[](
    vec2(0.0, 0.0), // Texture coordinate for bottom-left
    vec2(2.0, 0.0), // Texture coordinate for bottom-right
    vec2(0.0, 2.0)  // Texture coordinate for top-left
    );

    fragTexCoord = texCoords[gl_VertexIndex];
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}