#version 430

in vec2 texCoord;
out vec4 out_color;

layout(location = 0) uniform float iTime;
layout(location = 1) uniform vec2 iResolution;
layout(location = 3) uniform vec2 screenResolution;

// --- tool ---

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
  // Normalized pixel coordinates (from 0 to 1)
  vec2 uv = fragCoord/iResolution.xy;

  vec3 base = vec3(0.0, 2.0, 4.0); // !! color
  float x = 0.5; // !! slider
  vec3 col = 0.5 + x*cos(iTime+uv.xyx + base);

  // Output to screen
  fragColor = vec4(col,1.0);
}

// --- tool ---

void main() {
  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);
  mainImage(col, gl_FragCoord.xy);
  out_color = col;
}
