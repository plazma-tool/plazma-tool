#version 430
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 nor;
layout(location = 2) in vec2 tex;

out vec3 v_position;
out vec3 v_normal;
out vec2 v_texcoord;

layout(location = 0) uniform mat4 model;
layout(location = 1) uniform mat4 view;
layout(location = 2) uniform mat4 projection;

void main() {
  v_position = pos;
  v_normal = nor;
  gl_Position = projection * view * vec4(v_position * 0.2, 1.0);
  v_texcoord = tex;
}
