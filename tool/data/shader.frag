#version 430

out vec4 out_color;

uniform float iGlobalTime;
//uniform vec2 iResolution;

//uniform vec3 bg_color;
vec3 bg_color = vec3(0.1, 0.2, 0.3);

void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
  //vec2 uv = -1.0 + 2.0 * fragCoord.xy / iResolution.xy;
  //uv.x *= iResolution.x / iResolution.y;

  // background
  vec3 col = bg_color * sin(iGlobalTime * 0.001);

  fragColor = vec4(col, 1.0);
}

void main() {
  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);
  mainImage(col, gl_FragCoord.xy);
  out_color = col;
}

