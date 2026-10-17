#version 450
layout(location=0) in vec2 pos;
layout(location=1) in vec2 uv;
layout(location=2) in vec4 col;
layout(location=0) out vec2 v_uv;
layout(location=1) out vec4 v_col;

// Set these to your draw_image / display_size
const vec2 DISPLAY_SIZE = vec2(1280.0, 720.0);

void main() {
    v_uv = uv;
    v_col = col;
    vec2 scale = 2.0 / DISPLAY_SIZE;
    vec2 translate = vec2(-1.0, -1.0);
    gl_Position = vec4(pos * scale + translate, 0.0, 1.0);
}