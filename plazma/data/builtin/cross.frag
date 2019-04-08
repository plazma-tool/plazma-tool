#version 430

// texCoord is the texture coordinate on the quad in the 0.0 .. 1.0 range, where
// (0, 0) is the bottom-left corner.
in vec2 texCoord;
out vec4 out_color;

layout(location = 0) uniform float iTime;
layout(location = 1) uniform vec2 iResolution;
layout(location = 2) uniform vec2 screenResolution;

layout (binding = 0) uniform sampler2D screenTexture;

// Draws a cross in the middle of the screen, blending with screenTexture, which
// was the previous render step.

void main() {
  vec2 uv = -1.0 + 2.0 * gl_FragCoord.xy / iResolution.xy;
  uv.x *= iResolution.x / iResolution.y;

  // texture framebuffer resolution is the same as the window resolution
  // texCoord is 0.0 .. 1.0 at the size of the quad, which remains screen size
  vec2 tex_uv = texCoord * (screenResolution / iResolution);
  vec3 col = vec3(texture(screenTexture, tex_uv));

  float n = smoothstep(-0.02, 0.0, uv.x) - smoothstep(0.0, 0.02, uv.x);
  n += smoothstep(-0.02, 0.0, uv.y) - smoothstep(0.0, 0.02, uv.y);
  col += vec3(1., 0., 0.) * n;

  out_color = vec4(col, 1.0);
}
