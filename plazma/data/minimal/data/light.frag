#version 430
out vec4 out_color;

in vec3 fragPos;
in vec3 normal;
in vec3 texCoord;

void main() {
  out_color = vec4(vec3(1.0), 1.0);
}

