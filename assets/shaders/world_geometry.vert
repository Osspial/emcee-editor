#version 140

in vec3 pos;
in vec3 face_color;

uniform mat4 transform_matrix;

out vec3 frag_color;

void main() {
    gl_Position = transform_matrix * vec4(pos, 1.0);
    frag_color = face_color;
}
