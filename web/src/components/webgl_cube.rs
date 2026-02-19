use gloo::timers::callback::Interval;
use js_sys::Date;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, WebGlShader};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WebglCubeProps {
    pub collapsed: bool,
}

#[function_component(WebglCube)]
pub fn webgl_cube(WebglCubeProps { collapsed }: &WebglCubeProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        use_effect_with((), move |_| {
            let mut tick: Option<Interval> = None;

            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>()
                && let Ok(Some(ctx)) = canvas.get_context("webgl")
                && let Ok(gl) = ctx.dyn_into::<WebGlRenderingContext>()
                && let Some(program) = setup_program(&gl)
                && let Some(buffer) = gl.create_buffer()
            {
                gl.use_program(Some(&program));
                let vertices = cube_vertices();
                gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
                let vert_array = js_sys::Float32Array::from(vertices.as_slice());
                gl.buffer_data_with_array_buffer_view(
                    WebGlRenderingContext::ARRAY_BUFFER,
                    &vert_array,
                    WebGlRenderingContext::STATIC_DRAW,
                );

                let pos_loc = gl.get_attrib_location(&program, "a_position");
                let color_loc = gl.get_attrib_location(&program, "a_color");

                if pos_loc >= 0 && color_loc >= 0 {
                    let stride = (6 * std::mem::size_of::<f32>()) as i32;
                    gl.vertex_attrib_pointer_with_i32(
                        pos_loc as u32,
                        3,
                        WebGlRenderingContext::FLOAT,
                        false,
                        stride,
                        0,
                    );
                    gl.enable_vertex_attrib_array(pos_loc as u32);
                    gl.vertex_attrib_pointer_with_i32(
                        color_loc as u32,
                        3,
                        WebGlRenderingContext::FLOAT,
                        false,
                        stride,
                        (3 * std::mem::size_of::<f32>()) as i32,
                    );
                    gl.enable_vertex_attrib_array(color_loc as u32);

                    gl.enable(WebGlRenderingContext::DEPTH_TEST);
                    gl.clear_color(0.0, 0.0, 0.0, 0.0);

                    if let Some(u_matrix) = gl.get_uniform_location(&program, "u_matrix")
                        && let Some(u_time) = gl.get_uniform_location(&program, "u_time")
                    {
                        let start = Date::now();
                        tick = Some(Interval::new(16, move || {
                            let t = ((Date::now() - start) / 1000.0) as f32;
                            let w = canvas.width() as i32;
                            let h = canvas.height() as i32;
                            gl.viewport(0, 0, w, h);
                            gl.clear(
                                WebGlRenderingContext::COLOR_BUFFER_BIT
                                    | WebGlRenderingContext::DEPTH_BUFFER_BIT,
                            );

                            let aspect = if h > 0 { w as f32 / h as f32 } else { 1.0 };
                            let proj = perspective(50.0_f32.to_radians(), aspect, 0.1, 100.0);
                            let rot_x = rotation_x(t * 0.7);
                            let rot_y = rotation_y(t * 1.0);
                            let model =
                                multiply(translation(0.0, 0.0, -4.0), multiply(rot_y, rot_x));
                            let mvp = multiply(proj, model);

                            gl.uniform_matrix4fv_with_f32_array(Some(&u_matrix), false, &mvp);
                            gl.uniform1f(Some(&u_time), t);
                            gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 36);
                        }));
                    }
                }
            }

            move || drop(tick)
        });
    }

    let class = if *collapsed {
        classes!("cube-dock", "is-collapsed")
    } else {
        classes!("cube-dock")
    };

    html! {
        <div class={class}>
            <canvas ref={canvas_ref} class="cube-canvas" width="220" height="220" />
        </div>
    }
}

fn setup_program(gl: &WebGlRenderingContext) -> Option<WebGlProgram> {
    let vert_src = r#"
attribute vec3 a_position;
attribute vec3 a_color;
uniform mat4 u_matrix;
varying vec3 v_color;

void main() {
    v_color = a_color;
    gl_Position = u_matrix * vec4(a_position, 1.0);
}
"#;
    let frag_src = r#"
precision mediump float;
varying vec3 v_color;
uniform float u_time;

void main() {
    vec3 pulse = vec3(0.7, 1.1, 1.6) * u_time;
    vec3 rgb = abs(sin(v_color + pulse));
    gl_FragColor = vec4(rgb, 1.0);
}
"#;

    let vert = compile_shader(gl, WebGlRenderingContext::VERTEX_SHADER, vert_src)?;
    let frag = compile_shader(gl, WebGlRenderingContext::FRAGMENT_SHADER, frag_src)?;
    link_program(gl, &vert, &frag)
}

fn compile_shader(gl: &WebGlRenderingContext, kind: u32, source: &str) -> Option<WebGlShader> {
    let shader = gl.create_shader(kind)?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Some(shader)
    } else {
        None
    }
}

fn link_program(
    gl: &WebGlRenderingContext,
    vert: &WebGlShader,
    frag: &WebGlShader,
) -> Option<WebGlProgram> {
    let program = gl.create_program()?;
    gl.attach_shader(&program, vert);
    gl.attach_shader(&program, frag);
    gl.link_program(&program);
    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Some(program)
    } else {
        None
    }
}

fn cube_vertices() -> Vec<f32> {
    vec![
        // front
        -1.0, -1.0, 1.0, 1.0, 0.1, 0.1, 1.0, -1.0, 1.0, 1.0, 0.1, 0.1, 1.0, 1.0, 1.0, 1.0, 0.1, 0.1,
        -1.0, -1.0, 1.0, 1.0, 0.1, 0.1, 1.0, 1.0, 1.0, 1.0, 0.1, 0.1, -1.0, 1.0, 1.0, 1.0, 0.1,
        0.1, // back
        -1.0, -1.0, -1.0, 0.1, 0.8, 1.0, -1.0, 1.0, -1.0, 0.1, 0.8, 1.0, 1.0, 1.0, -1.0, 0.1, 0.8,
        1.0, -1.0, -1.0, -1.0, 0.1, 0.8, 1.0, 1.0, 1.0, -1.0, 0.1, 0.8, 1.0, 1.0, -1.0, -1.0, 0.1,
        0.8, 1.0, // left
        -1.0, -1.0, -1.0, 0.1, 1.0, 0.2, -1.0, -1.0, 1.0, 0.1, 1.0, 0.2, -1.0, 1.0, 1.0, 0.1, 1.0,
        0.2, -1.0, -1.0, -1.0, 0.1, 1.0, 0.2, -1.0, 1.0, 1.0, 0.1, 1.0, 0.2, -1.0, 1.0, -1.0, 0.1,
        1.0, 0.2, // right
        1.0, -1.0, -1.0, 0.9, 0.7, 0.1, 1.0, 1.0, -1.0, 0.9, 0.7, 0.1, 1.0, 1.0, 1.0, 0.9, 0.7,
        0.1, 1.0, -1.0, -1.0, 0.9, 0.7, 0.1, 1.0, 1.0, 1.0, 0.9, 0.7, 0.1, 1.0, -1.0, 1.0, 0.9,
        0.7, 0.1, // top
        -1.0, 1.0, -1.0, 0.8, 0.2, 1.0, -1.0, 1.0, 1.0, 0.8, 0.2, 1.0, 1.0, 1.0, 1.0, 0.8, 0.2,
        1.0, -1.0, 1.0, -1.0, 0.8, 0.2, 1.0, 1.0, 1.0, 1.0, 0.8, 0.2, 1.0, 1.0, 1.0, -1.0, 0.8,
        0.2, 1.0, // bottom
        -1.0, -1.0, -1.0, 0.1, 0.9, 0.6, 1.0, -1.0, -1.0, 0.1, 0.9, 0.6, 1.0, -1.0, 1.0, 0.1, 0.9,
        0.6, -1.0, -1.0, -1.0, 0.1, 0.9, 0.6, 1.0, -1.0, 1.0, 0.1, 0.9, 0.6, -1.0, -1.0, 1.0, 0.1,
        0.9, 0.6,
    ]
}

fn multiply(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut out = [0.0; 16];
    for c in 0..4 {
        for r in 0..4 {
            out[c * 4 + r] = a[r] * b[c * 4]
                + a[4 + r] * b[c * 4 + 1]
                + a[8 + r] * b[c * 4 + 2]
                + a[12 + r] * b[c * 4 + 3];
        }
    }
    out
}

fn perspective(fovy_rad: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fovy_rad * 0.5).tan();
    let nf = 1.0 / (near - far);
    [
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        (far + near) * nf,
        -1.0,
        0.0,
        0.0,
        (2.0 * far * near) * nf,
        0.0,
    ]
}

fn rotation_x(rad: f32) -> [f32; 16] {
    let c = rad.cos();
    let s = rad.sin();
    [
        1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn rotation_y(rad: f32) -> [f32; 16] {
    let c = rad.cos();
    let s = rad.sin();
    [
        c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn translation(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}
