use gloo::timers::callback::Interval;
use js_sys::Date;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{HtmlCanvasElement, MouseEvent, WebGlProgram, WebGlRenderingContext, WebGlShader};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WebglCubeProps {
    pub collapsed: bool,
}

#[derive(Default)]
struct MotionState {
    rot_x: f32,
    rot_y: f32,
    vel_x: f32,
    vel_y: f32,
}

#[function_component(WebglCube)]
pub fn webgl_cube(WebglCubeProps { collapsed }: &WebglCubeProps) -> Html {
    let canvas_ref = use_node_ref();
    let collapsed_flag = use_mut_ref(|| *collapsed);

    {
        let collapsed_flag = collapsed_flag.clone();
        use_effect_with(*collapsed, move |is_collapsed| {
            *collapsed_flag.borrow_mut() = *is_collapsed;
            || ()
        });
    }

    {
        let canvas_ref = canvas_ref.clone();
        let collapsed_flag = collapsed_flag.clone();
        use_effect_with((), move |_| {
            let mut tick: Option<Interval> = None;
            let mut on_mouse_move: Option<Closure<dyn FnMut(MouseEvent)>> = None;
            let mut on_mouse_leave: Option<Closure<dyn FnMut(MouseEvent)>> = None;

            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>()
                && let Ok(Some(ctx)) = canvas.get_context("webgl")
                && let Ok(gl) = ctx.dyn_into::<WebGlRenderingContext>()
                && let Some(program) = setup_program(&gl)
                && let Some(buffer) = gl.create_buffer()
            {
                gl.use_program(Some(&program));
                let mut vertices = cube_vertices();
                let cube_vertex_count = (vertices.len() / 6) as i32;
                vertices.extend_from_slice(&axes_vertices());
                let axis_vertex_count = ((vertices.len() / 6) as i32) - cube_vertex_count;
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
                        let motion = Rc::new(RefCell::new(MotionState {
                            vel_x: 0.7,
                            vel_y: 1.0,
                            ..MotionState::default()
                        }));
                        let motion_for_mouse = motion.clone();
                        let canvas_for_mouse = canvas.clone();
                        let collapsed_for_mouse = collapsed_flag.clone();

                        on_mouse_move = Some(Closure::wrap(Box::new(move |event: MouseEvent| {
                            if *collapsed_for_mouse.borrow() {
                                return;
                            }

                            let rect = canvas_for_mouse.get_bounding_client_rect();
                            let cx = rect.left() + rect.width() * 0.5;
                            let cy = rect.top() + rect.height() * 0.5;
                            let half_w = (rect.width() * 0.5).max(1.0);
                            let half_h = (rect.height() * 0.5).max(1.0);
                            let nx = ((event.client_x() as f64 - cx) / half_w).clamp(-1.0, 1.0);
                            let ny = ((event.client_y() as f64 - cy) / half_h).clamp(-1.0, 1.0);

                            let mut state = motion_for_mouse.borrow_mut();
                            state.vel_y = (nx as f32) * 2.4;
                            state.vel_x = (-(ny as f32)) * 2.4;
                        })
                            as Box<dyn FnMut(_)>));
                        if let Some(handler) = on_mouse_move.as_ref() {
                            let _ = canvas.add_event_listener_with_callback(
                                "mousemove",
                                handler.as_ref().unchecked_ref(),
                            );
                        }

                        let motion_for_leave = motion.clone();
                        on_mouse_leave = Some(Closure::wrap(Box::new(move |_event: MouseEvent| {
                            let mut state = motion_for_leave.borrow_mut();
                            state.vel_x = 0.7;
                            state.vel_y = 1.0;
                        })
                            as Box<dyn FnMut(_)>));
                        if let Some(handler) = on_mouse_leave.as_ref() {
                            let _ = canvas.add_event_listener_with_callback(
                                "mouseleave",
                                handler.as_ref().unchecked_ref(),
                            );
                        }

                        let start = Date::now();
                        let mut last = start;
                        tick = Some(Interval::new(16, move || {
                            let now = Date::now();
                            let dt = ((now - last) / 1000.0) as f32;
                            last = now;

                            let t = ((now - start) / 1000.0) as f32;
                            let w = canvas.width() as i32;
                            let h = canvas.height() as i32;
                            gl.viewport(0, 0, w, h);
                            gl.clear(
                                WebGlRenderingContext::COLOR_BUFFER_BIT
                                    | WebGlRenderingContext::DEPTH_BUFFER_BIT,
                            );

                            let mut state = motion.borrow_mut();
                            state.rot_x += state.vel_x * dt;
                            state.rot_y += state.vel_y * dt;

                            let aspect = if h > 0 { w as f32 / h as f32 } else { 1.0 };
                            let proj = perspective(50.0_f32.to_radians(), aspect, 0.1, 100.0);
                            let rot_x = rotation_x(state.rot_x);
                            let rot_y = rotation_y(state.rot_y);
                            let model =
                                multiply(translation(0.0, 0.0, -4.0), multiply(rot_y, rot_x));
                            let mvp = multiply(proj, model);

                            gl.uniform_matrix4fv_with_f32_array(Some(&u_matrix), false, &mvp);
                            gl.uniform1f(Some(&u_time), t);
                            gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, cube_vertex_count);
                            gl.uniform1f(Some(&u_time), 0.0);
                            gl.draw_arrays(
                                WebGlRenderingContext::LINES,
                                cube_vertex_count,
                                axis_vertex_count,
                            );
                        }));
                    }
                }
            }

            move || {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    if let Some(handler) = on_mouse_move.as_ref() {
                        let _ = canvas.remove_event_listener_with_callback(
                            "mousemove",
                            handler.as_ref().unchecked_ref(),
                        );
                    }
                    if let Some(handler) = on_mouse_leave.as_ref() {
                        let _ = canvas.remove_event_listener_with_callback(
                            "mouseleave",
                            handler.as_ref().unchecked_ref(),
                        );
                    }
                }
                drop(tick);
                drop(on_mouse_move);
                drop(on_mouse_leave);
            }
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
    let vert_src = include_str!("shaders/cube.vert");
    let frag_src = include_str!("shaders/cube.frag");

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

fn axes_vertices() -> Vec<f32> {
    let l = 1.7;
    vec![
        // X axis (red)
        -l, 0.0, 0.0, 1.0, 0.0, 0.0, l, 0.0, 0.0, 1.0, 0.0, 0.0, // Y axis (green)
        0.0, -l, 0.0, 0.0, 1.0, 0.0, 0.0, l, 0.0, 0.0, 1.0, 0.0, // Z axis (blue)
        0.0, 0.0, -l, 0.0, 0.0, 1.0, 0.0, 0.0, l, 0.0, 0.0, 1.0,
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
