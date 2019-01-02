#version 430

out vec4 out_color;

layout(location = 0) uniform float iTime;
layout(location = 1) uniform vec2 iResolution;
layout(location = 3) uniform vec2 screenResolution;

// --- tool ---

// Draws a circle in the middle of the screen.

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
  // Normalize the scale with the resolution so that a rectangle (screen-shaped)
  // quad preserves the square aspect ratio, and move the range to -1.0 .. 1.0
  // where (0, 0) is the center of the quad.
  vec2 uv = -1.0 + 2.0 * fragCoord.xy / iResolution.xy;
  uv.x *= iResolution.x / iResolution.y;

  vec3 base = vec3(0.1, 0.2, 0.3); // !! color
  vec3 circle = vec3(0., 0., 1.); // !! color
  float radius = 0.3; // !! slider

  vec3 col = base + circle * (1.0 - smoothstep(radius, radius + 0.01, distance(uv, vec2(0.0))));

  fragColor = vec4(col, 1.0);
}

// --- tool ---

void main() {
  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);
  // Pass in gl_FragCoord so that after normalization, (0, 0) will always be the
  // center of the window.
  mainImage(col, gl_FragCoord.xy);
  out_color = col;
}
