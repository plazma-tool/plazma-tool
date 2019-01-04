#version 430
out vec4 f_color;

in vec3 v_position;
in vec3 v_normal;
in vec2 v_texcoord;

layout(location = 4) uniform float iGlobalTime;
layout(location = 5) uniform vec3 g_light_position;

void main() {
  float lum = max(dot(normalize(v_normal), normalize(g_light_position)), 0.0);
  vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
  f_color = vec4(color, 1.0);
}
