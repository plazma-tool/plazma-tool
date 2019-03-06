#version 430
out vec4 out_color;

in vec3 fragPos;
in vec3 normal;
in vec2 texCoord;

// Locations 0, 1, 2, 3 are always bound when drawing a polygon mesh

//layout(location = 0) uniform mat4 model;
//layout(location = 1) uniform mat4 view;
//layout(location = 2) uniform mat4 projection;
layout(location = 3) uniform vec3 view_pos;

// Further locations are bound with layout_to_vars.

layout(location = 4) uniform float iTime; // Time
layout(location = 5) uniform vec2 iResolution; // Window_Width, _Height
layout(location = 6) uniform vec2 screenResolution; // Screen_Width, _Height
layout(location = 7) uniform vec3 g_light_position; // Light_Pos_X, _Y, _Z,

layout (binding = 0) uniform sampler2D objTexture;

void main() {

  vec2 tex_uv = texCoord * (screenResolution / iResolution);
  vec3 material_diffuse = vec3(texture(objTexture, tex_uv));

  vec3 result = material_diffuse;

  out_color = vec4(result, 1.0);
}

