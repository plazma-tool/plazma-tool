#version 430
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 nor;
layout(location = 2) in vec2 tex;

out vec3 fragPos;
out vec3 normal;
out vec2 texCoord;

layout(location = 0) uniform mat4 model;
layout(location = 1) uniform mat4 view;
layout(location = 2) uniform mat4 projection;

void main() {
  fragPos = vec3(model * vec4(pos, 1.0));
  normal = mat3(transpose(inverse(model))) * nor;

  gl_Position = projection * view * vec4(fragPos, 1.0);
  texCoord = tex;
}
