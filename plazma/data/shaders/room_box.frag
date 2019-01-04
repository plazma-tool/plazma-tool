#version 430
out vec4 out_color;

in vec3 fragPos;
in vec3 normal;
in vec2 texCoord;

layout(location = 3) uniform vec3 viewPos;

layout(location = 4) uniform float iGlobalTime;
layout(location = 5) uniform vec3 g_light_position;

layout(location = 6) uniform vec2 iResolution;
layout(location = 7) uniform vec2 screenResolution;

layout (binding = 0) uniform sampler2D objTexture;

void main() {
  vec3 ambient = vec3(0.001);
  vec3 light_diffuse = vec3(0.3);
  vec3 light_specular = vec3(0.2);
  vec3 material_specular = vec3(1.0);

  vec3 material_diffuse = vec3(0.41, 0.58, 0.70);
  //vec2 tex_uv = texCoord * (screenResolution / iResolution);
  //vec3 material_diffuse = vec3(texture(objTexture, tex_uv));

  // diffuse
  vec3 norm = normalize(-normal);
  vec3 lightDir = normalize(g_light_position - fragPos);
  float diff = max(dot(norm, lightDir), 0.0);
  vec3 diffuse = light_diffuse * (diff * material_diffuse);

  // specular

  //vec3 viewDir = normalize(viewPos - fragPos);
  //vec3 reflectDir = reflect(-lightDir, norm);
  //float spec = pow(max(dot(viewDir, reflectDir), 0.0), 24.0);
  //vec3 specular = light_specular * (spec * material_specular);
  vec3 specular = vec3(0.0);

  vec3 result = ambient + diffuse + specular;

  out_color = vec4(result, 1.0);
}

