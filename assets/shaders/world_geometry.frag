#version 140

in vec3 frag_color;

out vec4 out_color;

void main() {
    out_color = vec4(frag_color/* * gl_FragCoord.z * gl_FragCoord.w * 5.0*/, 1.0);
}
