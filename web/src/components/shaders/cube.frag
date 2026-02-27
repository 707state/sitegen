precision mediump float;                      
varying vec3 v_color;                         
uniform float u_time;                         
                                              
void main() {                                 
    vec3 pulse = vec3(0.7, 1.1, 1.6) * u_time;
    vec3 rgb = abs(sin(v_color + pulse));     
    gl_FragColor = vec4(rgb, 1.0);            
}                                             