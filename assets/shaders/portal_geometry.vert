#version 140

in vec3 pos;

uniform mat4 transform_matrix;

out vec3 frag_color;

void main() {
    gl_Position = transform_matrix * vec4(pos, 1.0);
    frag_color = vec3(1.0, 0.0, 1.0);
}
