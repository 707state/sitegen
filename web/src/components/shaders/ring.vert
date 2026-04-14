attribute vec3 a_position;
attribute vec3 a_normal;
attribute vec2 a_uv;

uniform mat4 u_mvp;
uniform mat4 u_model_view;

varying vec3 v_view_pos;
varying vec3 v_normal;
varying vec2 v_uv;

void main() {
    vec4 view_pos = u_model_view * vec4(a_position, 1.0);
    v_view_pos = view_pos.xyz;
    v_normal = normalize((u_model_view * vec4(a_normal, 0.0)).xyz);
    v_uv = a_uv;
    gl_Position = u_mvp * vec4(a_position, 1.0);
}
