#version 430

in vec2 texCoord;
out vec4 out_color;

layout(location = 0) uniform vec2 iResolution;
layout(location = 1) uniform vec2 screenResolution;

layout (binding = 0) uniform sampler2D screenTexture;

void main() {
  vec2 tex_uv = texCoord * (screenResolution / iResolution);
  vec3 col = vec3(texture(screenTexture, tex_uv));
  out_color = vec4(col, 1.0);
}
