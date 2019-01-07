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
  vec3 ambient = vec3(0.1);
  vec3 light_diffuse = vec3(0.8, 0.0, 0.8) + vec3(sin(iTime), 0., 0.);
  vec3 light_specular = vec3(1.0);
  vec3 material_specular = vec3(0.0, 0.0, 0.9);

  //vec3 material_diffuse = vec3(0.9, 0.0, 0.0);
  vec2 tex_uv = texCoord * (screenResolution / iResolution);
  vec3 material_diffuse = vec3(texture(objTexture, tex_uv));

  // diffuse
  vec3 norm = normalize(normal);
  vec3 lightDir = normalize(g_light_position - fragPos);
  float diff = max(dot(norm, lightDir), 0.0);
  vec3 diffuse = light_diffuse * (diff * material_diffuse);

  // specular
  vec3 viewDir = normalize(view_pos - fragPos);
  vec3 reflectDir = reflect(-lightDir, norm);
  float spec = pow(max(dot(viewDir, reflectDir), 0.0), 24.0);
  vec3 specular = light_specular * (spec * material_specular);

  vec3 result = ambient + diffuse + specular;

  out_color = vec4(result, 1.0);
}

