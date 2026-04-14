precision mediump float;

varying vec3 v_view_pos;
varying vec3 v_normal;
varying vec2 v_uv;

uniform float u_time;
uniform vec3 u_color;
uniform vec3 u_accent;
uniform float u_kind;
uniform float u_energy;
uniform float u_focus;
uniform float u_gap;

float hash21(vec2 p) {
    p = fract(p * vec2(234.34, 435.345));
    p += dot(p, p + 34.23);
    return fract(p.x * p.y);
}

void main() {
    vec3 normal = normalize(v_normal);
    vec3 view_dir = normalize(-v_view_pos);
    vec3 light_dir = normalize(vec3(-0.5, 0.8, 0.4));
    float diffuse = max(dot(normal, light_dir), 0.0);
    float fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 2.4);
    float pulse = 0.5 + 0.5 * sin(u_time * 0.5 + v_uv.x * 6.28318);
    vec3 color = u_color;
    float alpha = 1.0;

    if (u_kind < 0.5) {
        float bands = 0.5 + 0.5 * sin(v_uv.y * 24.0 + u_time * 0.35);
        float storm = hash21(v_uv * 9.0 + u_time * 0.03);
        color = mix(u_color, u_accent, bands * 0.45 + storm * 0.18);
        color *= 0.38 + diffuse * 0.88;
        color += u_accent * fresnel * 0.48;
        alpha = 0.92 * u_energy;
    } else if (u_kind < 1.5) {
        float lane = smoothstep(0.1, 0.42, 1.0 - abs(v_uv.y - 0.5) * 2.0);
        float spark = hash21(vec2(v_uv.x * 72.0, floor(v_uv.y * 18.0) + floor(u_time * 2.0)));
        color = mix(u_color * 0.55, u_accent, pulse * 0.35 + spark * 0.35);
        color += u_accent * fresnel * 0.65;
        alpha = lane * (0.26 + pulse * 0.18) * u_energy;
    } else if (u_kind < 2.5) {
        float halo = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.4);
        color = mix(u_color, u_accent, pulse * 0.5);
        color += u_accent * halo * 1.2;
        alpha = (0.18 + halo * 0.55) * u_energy;
    } else {
        float lane = smoothstep(0.16, 0.46, 1.0 - abs(v_uv.y - 0.5) * 2.0);
        float focus_dist = abs(fract(v_uv.x - u_focus + 0.5) - 0.5);
        float gap = smoothstep(u_gap, u_gap + 0.028, focus_dist);
        color = mix(u_color * 0.7, u_accent, pulse * 0.22);
        color += u_accent * fresnel * 0.25;
        alpha = lane * gap * (0.22 + pulse * 0.08) * u_energy;
    }

    gl_FragColor = vec4(color, alpha);
}
