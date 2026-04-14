use crate::components::TocItem;
use chrono::Datelike;
use gloo::timers::callback::Interval;
use js_sys::Date;
use std::collections::{BTreeMap, HashSet};
use std::f32::consts::PI;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{
    HtmlCanvasElement, MouseEvent, TouchEvent, WebGlBuffer, WebGlProgram, WebGlRenderingContext,
    WebGlShader, WheelEvent,
};
use yew::prelude::*;

#[derive(Clone)]
struct MonthSlot {
    month: u32,
    post_indices: Vec<usize>,
    angle: f32,
}

#[derive(Clone)]
struct YearPlanet {
    year: i32,
    orbit_radius: f32,
    orbit_phase: f32,
    orbit_speed: f32,
    orbit_tilt: f32,
    radius: f32,
    ring_radius: f32,
    ring_tilt: f32,
    color: [f32; 3],
    accent: [f32; 3],
    months: Vec<MonthSlot>,
}

fn build_solar_system(toc_items: &[TocItem]) -> Vec<YearPlanet> {
    let mut by_year: BTreeMap<i32, BTreeMap<u32, Vec<usize>>> = BTreeMap::new();
    for (idx, item) in toc_items.iter().enumerate() {
        by_year
            .entry(item.date.year())
            .or_default()
            .entry(item.date.month())
            .or_default()
            .push(idx);
    }

    let years: Vec<i32> = by_year.keys().copied().collect();
    years
        .into_iter()
        .enumerate()
        .map(|(i, year)| {
            let months_map = &by_year[&year];
            let post_count = months_map.values().map(Vec::len).sum::<usize>() as f32;
            let seed = (year as f32 * 0.0713).sin() * 0.5 + 0.5;
            let radius = 0.72 + post_count.sqrt() * 0.06;
            let ring_radius = radius + 2.05 + months_map.len() as f32 * 0.045;
            let ring_tilt = 0.22 + seed * 0.42;
            let color = [0.22 + seed * 0.18, 0.46 + seed * 0.18, 0.74 + seed * 0.15];
            let accent = [
                0.80 + seed * 0.12,
                0.63 + (1.0 - seed) * 0.17,
                0.35 + seed * 0.16,
            ];
            let months = months_map
                .iter()
                .map(|(&month, posts)| MonthSlot {
                    month,
                    post_indices: posts.clone(),
                    angle: (month as f32 - 1.0) / 12.0 * PI * 2.0 - PI * 0.5,
                })
                .collect::<Vec<_>>();

            YearPlanet {
                year,
                orbit_radius: 5.4 + i as f32 * 3.2,
                orbit_phase: seed * PI * 2.0 + i as f32 * 0.8,
                orbit_speed: 0.08 + 0.12 / (1.0 + i as f32 * 0.55),
                orbit_tilt: -0.08 + seed * 0.16,
                radius,
                ring_radius,
                ring_tilt,
                color,
                accent,
                months,
            }
        })
        .collect()
}

struct CamState {
    rot_x: f32,
    rot_y: f32,
    zoom: f32,
    vel_x: f32,
    vel_y: f32,
    is_dragging: bool,
    drag_x: f32,
    drag_y: f32,
    pinch_distance: Option<f32>,
}

impl Default for CamState {
    fn default() -> Self {
        Self {
            rot_x: -PI * 0.41,
            rot_y: 0.0,
            zoom: 27.0,
            vel_x: 0.0,
            vel_y: 0.08,
            is_dragging: false,
            drag_x: 0.0,
            drag_y: 0.0,
            pinch_distance: None,
        }
    }
}

#[derive(Clone, PartialEq)]
struct MonthPostLink {
    title: String,
    path: String,
}

#[derive(Clone, PartialEq)]
struct LabelInfo {
    key: String,
    kind: LabelKind,
    x: f64,
    y: f64,
    opacity: f64,
    width: f64,
    height: f64,
    anchor_left: bool,
    title: String,
    sub: String,
    posts: Vec<MonthPostLink>,
}

#[derive(Clone, PartialEq)]
enum LabelKind {
    Year,
    Month,
}

#[derive(Properties, PartialEq)]
pub struct WebglRingProps {
    pub toc_items: Vec<TocItem>,
    pub on_open_post: Callback<String>,
}

#[function_component(WebglRing)]
pub fn webgl_ring(
    WebglRingProps {
        toc_items,
        on_open_post,
    }: &WebglRingProps,
) -> Html {
    let canvas_ref = use_node_ref();
    let labels = use_state(Vec::<LabelInfo>::new);
    let expanded_months = use_state(HashSet::<String>::new);
    let cam = use_mut_ref(CamState::default);

    let system = {
        let items = toc_items.clone();
        use_memo(items, |items| build_solar_system(items))
    };

    {
        let canvas_ref = canvas_ref.clone();
        let cam = cam.clone();
        let labels = labels.clone();
        let expanded_months = expanded_months.clone();
        let system = system.clone();
        let toc_items_for_effect = toc_items.clone();

        use_effect_with((), move |_| {
            let mut tick: Option<Interval> = None;
            let mut on_mousedown: Option<Closure<dyn FnMut(MouseEvent)>> = None;
            let mut on_mousemove: Option<Closure<dyn FnMut(MouseEvent)>> = None;
            let mut on_mouseup: Option<Closure<dyn FnMut(MouseEvent)>> = None;
            let mut on_mouseleave: Option<Closure<dyn FnMut(MouseEvent)>> = None;
            let mut on_wheel: Option<Closure<dyn FnMut(WheelEvent)>> = None;
            let mut on_touchstart: Option<Closure<dyn FnMut(TouchEvent)>> = None;
            let mut on_touchmove: Option<Closure<dyn FnMut(TouchEvent)>> = None;
            let mut on_touchend: Option<Closure<dyn FnMut(TouchEvent)>> = None;

            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>()
                && let Ok(Some(ctx)) = canvas.get_context("webgl")
                && let Ok(gl) = ctx.dyn_into::<WebGlRenderingContext>()
                && let Some(program) = setup_program(&gl)
            {
                gl.use_program(Some(&program));
                gl.enable(WebGlRenderingContext::DEPTH_TEST);
                gl.enable(WebGlRenderingContext::BLEND);
                gl.blend_func(WebGlRenderingContext::SRC_ALPHA, WebGlRenderingContext::ONE);

                let u_mvp = gl.get_uniform_location(&program, "u_mvp");
                let u_model_view = gl.get_uniform_location(&program, "u_model_view");
                let u_time = gl.get_uniform_location(&program, "u_time");
                let u_color = gl.get_uniform_location(&program, "u_color");
                let u_accent = gl.get_uniform_location(&program, "u_accent");
                let u_kind = gl.get_uniform_location(&program, "u_kind");
                let u_energy = gl.get_uniform_location(&program, "u_energy");
                let u_focus = gl.get_uniform_location(&program, "u_focus");
                let u_gap = gl.get_uniform_location(&program, "u_gap");

                let torus = make_torus_buffers(&gl, 1.0, 0.075, 96, 20);
                let sphere = make_sphere_buffers(&gl, 1.0, 36, 28);

                let cam_md = cam.clone();
                let canvas_md = canvas.clone();
                on_mousedown = Some(Closure::wrap(Box::new(move |e: MouseEvent| {
                    let rect = canvas_md.get_bounding_client_rect();
                    let mut c = cam_md.borrow_mut();
                    c.is_dragging = true;
                    c.drag_x = e.client_x() as f32 - rect.left() as f32;
                    c.drag_y = e.client_y() as f32 - rect.top() as f32;
                    c.vel_x = 0.0;
                    c.vel_y = 0.0;
                    c.pinch_distance = None;
                }) as Box<dyn FnMut(_)>));

                let cam_mm = cam.clone();
                let canvas_mm = canvas.clone();
                on_mousemove = Some(Closure::wrap(Box::new(move |e: MouseEvent| {
                    let rect = canvas_mm.get_bounding_client_rect();
                    let mx = e.client_x() as f32 - rect.left() as f32;
                    let my = e.client_y() as f32 - rect.top() as f32;
                    let mut c = cam_mm.borrow_mut();
                    if c.is_dragging {
                        let dx = mx - c.drag_x;
                        let dy = my - c.drag_y;
                        let w = rect.width().max(1.0) as f32;
                        let h = rect.height().max(1.0) as f32;
                        c.rot_y += dx / w * PI * 1.4;
                        c.rot_x += dy / h * PI * 0.9;
                        c.rot_x = c.rot_x.clamp(-PI * 0.42, PI * 0.32);
                        c.vel_y = dx / w * PI * 4.8;
                        c.vel_x = dy / h * PI * 3.4;
                        c.drag_x = mx;
                        c.drag_y = my;
                    }
                }) as Box<dyn FnMut(_)>));

                let cam_mu = cam.clone();
                on_mouseup = Some(Closure::wrap(Box::new(move |_: MouseEvent| {
                    cam_mu.borrow_mut().is_dragging = false;
                }) as Box<dyn FnMut(_)>));

                let cam_ml = cam.clone();
                on_mouseleave = Some(Closure::wrap(Box::new(move |_: MouseEvent| {
                    cam_ml.borrow_mut().is_dragging = false;
                }) as Box<dyn FnMut(_)>));

                let cam_wh = cam.clone();
                on_wheel = Some(Closure::wrap(Box::new(move |e: WheelEvent| {
                    e.prevent_default();
                    let mut c = cam_wh.borrow_mut();
                    c.zoom = (c.zoom + e.delta_y() as f32 * 0.015).clamp(12.0, 42.0);
                }) as Box<dyn FnMut(_)>));

                let cam_ts = cam.clone();
                let canvas_ts = canvas.clone();
                on_touchstart = Some(Closure::wrap(Box::new(move |e: TouchEvent| {
                    e.prevent_default();
                    let rect = canvas_ts.get_bounding_client_rect();
                    let touches = e.touches();
                    let mut c = cam_ts.borrow_mut();
                    if touches.length() >= 2 {
                        if let (Some(a), Some(b)) = (touches.item(0), touches.item(1)) {
                            let dx = (a.client_x() - b.client_x()) as f32;
                            let dy = (a.client_y() - b.client_y()) as f32;
                            c.pinch_distance = Some((dx * dx + dy * dy).sqrt());
                            c.is_dragging = false;
                        }
                    } else if let Some(touch) = touches.item(0) {
                        c.is_dragging = true;
                        c.drag_x = touch.client_x() as f32 - rect.left() as f32;
                        c.drag_y = touch.client_y() as f32 - rect.top() as f32;
                        c.vel_x = 0.0;
                        c.vel_y = 0.0;
                        c.pinch_distance = None;
                    }
                }) as Box<dyn FnMut(_)>));

                let cam_tm = cam.clone();
                let canvas_tm = canvas.clone();
                on_touchmove = Some(Closure::wrap(Box::new(move |e: TouchEvent| {
                    e.prevent_default();
                    let rect = canvas_tm.get_bounding_client_rect();
                    let touches = e.touches();
                    let mut c = cam_tm.borrow_mut();
                    if touches.length() >= 2 {
                        if let (Some(a), Some(b)) = (touches.item(0), touches.item(1)) {
                            let dx = (a.client_x() - b.client_x()) as f32;
                            let dy = (a.client_y() - b.client_y()) as f32;
                            let distance = (dx * dx + dy * dy).sqrt();
                            if let Some(prev) = c.pinch_distance {
                                c.zoom = (c.zoom - (distance - prev) * 0.02).clamp(12.0, 42.0);
                            }
                            c.pinch_distance = Some(distance);
                            c.is_dragging = false;
                        }
                    } else if let Some(touch) = touches.item(0) {
                        let mx = touch.client_x() as f32 - rect.left() as f32;
                        let my = touch.client_y() as f32 - rect.top() as f32;
                        if c.is_dragging {
                            let dx = mx - c.drag_x;
                            let dy = my - c.drag_y;
                            let w = rect.width().max(1.0) as f32;
                            let h = rect.height().max(1.0) as f32;
                            c.rot_y += dx / w * PI * 1.3;
                            c.rot_x += dy / h * PI * 0.82;
                            c.rot_x = c.rot_x.clamp(-PI * 0.42, PI * 0.32);
                            c.vel_y = dx / w * PI * 4.2;
                            c.vel_x = dy / h * PI * 2.9;
                        }
                        c.drag_x = mx;
                        c.drag_y = my;
                        c.pinch_distance = None;
                    }
                }) as Box<dyn FnMut(_)>));

                let cam_te = cam.clone();
                on_touchend = Some(Closure::wrap(Box::new(move |e: TouchEvent| {
                    let touches = e.touches();
                    let mut c = cam_te.borrow_mut();
                    if touches.length() == 0 {
                        c.is_dragging = false;
                        c.pinch_distance = None;
                    } else if touches.length() == 1 {
                        c.pinch_distance = None;
                    }
                }) as Box<dyn FnMut(_)>));

                macro_rules! add_listener {
                    ($event:expr, $handler:expr) => {
                        if let Some(h) = $handler.as_ref() {
                            let _ = canvas.add_event_listener_with_callback(
                                $event,
                                h.as_ref().unchecked_ref(),
                            );
                        }
                    };
                }

                add_listener!("mousedown", on_mousedown);
                add_listener!("mousemove", on_mousemove);
                add_listener!("mouseup", on_mouseup);
                add_listener!("mouseleave", on_mouseleave);
                add_listener!("wheel", on_wheel);
                add_listener!("touchstart", on_touchstart);
                add_listener!("touchmove", on_touchmove);
                add_listener!("touchend", on_touchend);
                add_listener!("touchcancel", on_touchend);

                let start = Date::now();
                let mut last = start;
                let cam_tick = cam.clone();
                let labels_tick = labels.clone();
                let system_tick = system.clone();
                let toc_for_tick = toc_items_for_effect.clone();

                tick = Some(Interval::new(16, move || {
                    let now = Date::now();
                    let dt = ((now - last) / 1000.0) as f32;
                    let t = ((now - start) / 1000.0) as f32;
                    last = now;

                    let rect = canvas.get_bounding_client_rect();
                    let logical_w = rect.width().max(1.0);
                    let logical_h = rect.height().max(1.0);
                    let dpr = web_sys::window()
                        .map(|w| w.device_pixel_ratio())
                        .unwrap_or(1.0)
                        .clamp(1.0, 2.0);
                    let pixel_w = (logical_w * dpr).round() as u32;
                    let pixel_h = (logical_h * dpr).round() as u32;
                    if canvas.width() != pixel_w {
                        canvas.set_width(pixel_w);
                    }
                    if canvas.height() != pixel_h {
                        canvas.set_height(pixel_h);
                    }

                    gl.viewport(0, 0, pixel_w as i32, pixel_h as i32);
                    gl.clear_color(0.0, 0.0, 0.0, 0.0);
                    gl.clear(
                        WebGlRenderingContext::COLOR_BUFFER_BIT
                            | WebGlRenderingContext::DEPTH_BUFFER_BIT,
                    );

                    let (rot_x, rot_y, zoom) = {
                        let mut c = cam_tick.borrow_mut();
                        if !c.is_dragging {
                            c.rot_y += c.vel_y * dt;
                            c.rot_x += c.vel_x * dt;
                            c.rot_x = c.rot_x.clamp(-PI * 0.42, PI * 0.32);
                            let friction = 0.94_f32.powf(dt * 60.0);
                            c.vel_x *= friction;
                            c.vel_y *= friction;
                        }
                        (c.rot_x, c.rot_y, c.zoom)
                    };

                    let aspect = (logical_w / logical_h) as f32;
                    let proj = perspective(42.0_f32.to_radians(), aspect, 0.1, 180.0);
                    let view = translation(0.0, 0.6, -zoom);
                    let world = multiply(rotation_y(rot_y), rotation_x(rot_x));

                    let expanded = (*expanded_months).clone();
                    let mut next_labels = Vec::new();

                    if let Some((ref vb, ref ib, ic)) = sphere {
                        let sun_pulse = 1.0 + (t * 0.8).sin() * 0.035;
                        let sun_model = multiply(
                            view,
                            multiply(
                                world,
                                scale(1.95 * sun_pulse, 1.95 * sun_pulse, 1.95 * sun_pulse),
                            ),
                        );
                        let sun_mvp = multiply(proj, sun_model);
                        draw_indexed(
                            &gl,
                            &program,
                            vb,
                            ib,
                            ic,
                            &sun_model,
                            &sun_mvp,
                            t,
                            [0.96, 0.58, 0.16],
                            [1.0, 0.82, 0.38],
                            0.0,
                            1.12,
                            0.0,
                            0.0,
                            &u_mvp,
                            &u_model_view,
                            &u_time,
                            &u_color,
                            &u_accent,
                            &u_kind,
                            &u_energy,
                            &u_focus,
                            &u_gap,
                        );

                        let sun_glow_model = multiply(
                            view,
                            multiply(
                                world,
                                scale(2.7 * sun_pulse, 2.7 * sun_pulse, 2.7 * sun_pulse),
                            ),
                        );
                        let sun_glow_mvp = multiply(proj, sun_glow_model);
                        draw_indexed(
                            &gl,
                            &program,
                            vb,
                            ib,
                            ic,
                            &sun_glow_model,
                            &sun_glow_mvp,
                            t,
                            [0.98, 0.68, 0.22],
                            [1.0, 0.86, 0.54],
                            2.0,
                            0.84,
                            0.0,
                            0.0,
                            &u_mvp,
                            &u_model_view,
                            &u_time,
                            &u_color,
                            &u_accent,
                            &u_kind,
                            &u_energy,
                            &u_focus,
                            &u_gap,
                        );
                    }

                    for planet in system_tick.iter() {
                        let orbit_angle = t * planet.orbit_speed + planet.orbit_phase;
                        let orbit_track_rotation = multiply(
                            rotation_z(planet.orbit_tilt * 0.28),
                            rotation_x(planet.orbit_tilt),
                        );
                        let orbit_local = (
                            planet.orbit_radius * orbit_angle.cos(),
                            0.0,
                            planet.orbit_radius * orbit_angle.sin(),
                        );
                        let orbit_pos = transform_point(
                            &orbit_track_rotation,
                            orbit_local.0,
                            orbit_local.1,
                            orbit_local.2,
                        );
                        let orbit = translation(orbit_pos.0, orbit_pos.1, orbit_pos.2);
                        let model = multiply(world, orbit);
                        let model_view = multiply(view, model);

                        let orbit_track_model = multiply(
                            view,
                            multiply(
                                world,
                                multiply(
                                    orbit_track_rotation,
                                    scale(planet.orbit_radius, 0.22, planet.orbit_radius),
                                ),
                            ),
                        );
                        let orbit_track_mvp = multiply(proj, orbit_track_model);
                        if let Some((ref vb, ref ib, ic)) = torus {
                            let orbit_gap =
                                ((planet.radius * 2.4 + 0.45) / planet.orbit_radius / (PI * 2.0))
                                    .clamp(0.07, 0.16);
                            draw_indexed(
                                &gl,
                                &program,
                                vb,
                                ib,
                                ic,
                                &orbit_track_model,
                                &orbit_track_mvp,
                                t,
                                [0.18, 0.34, 0.5],
                                lighten(planet.color, 0.16),
                                3.0,
                                0.32,
                                orbit_angle / (PI * 2.0),
                                orbit_gap,
                                &u_mvp,
                                &u_model_view,
                                &u_time,
                                &u_color,
                                &u_accent,
                                &u_kind,
                                &u_energy,
                                &u_focus,
                                &u_gap,
                            );
                        }

                        let sphere_model = multiply(
                            model_view,
                            scale(planet.radius, planet.radius, planet.radius),
                        );
                        let sphere_mvp = multiply(proj, sphere_model);
                        if let Some((ref vb, ref ib, ic)) = sphere {
                            draw_indexed(
                                &gl,
                                &program,
                                vb,
                                ib,
                                ic,
                                &sphere_model,
                                &sphere_mvp,
                                t,
                                planet.color,
                                planet.accent,
                                0.0,
                                0.96,
                                0.0,
                                0.0,
                                &u_mvp,
                                &u_model_view,
                                &u_time,
                                &u_color,
                                &u_accent,
                                &u_kind,
                                &u_energy,
                                &u_focus,
                                &u_gap,
                            );

                            let glow_model = multiply(
                                model_view,
                                scale(
                                    planet.radius * 1.3,
                                    planet.radius * 1.3,
                                    planet.radius * 1.3,
                                ),
                            );
                            let glow_mvp = multiply(proj, glow_model);
                            draw_indexed(
                                &gl,
                                &program,
                                vb,
                                ib,
                                ic,
                                &glow_model,
                                &glow_mvp,
                                t,
                                planet.color,
                                planet.accent,
                                2.0,
                                0.5,
                                0.0,
                                0.0,
                                &u_mvp,
                                &u_model_view,
                                &u_time,
                                &u_color,
                                &u_accent,
                                &u_kind,
                                &u_energy,
                                &u_focus,
                                &u_gap,
                            );
                        }

                        let ring_rotation = multiply(
                            rotation_z(planet.ring_tilt * 0.6),
                            rotation_x(planet.ring_tilt),
                        );
                        let ring_model = multiply(
                            model_view,
                            multiply(
                                ring_rotation,
                                scale(planet.ring_radius, 1.0, planet.ring_radius),
                            ),
                        );
                        let ring_mvp = multiply(proj, ring_model);
                        if let Some((ref vb, ref ib, ic)) = torus {
                            draw_indexed(
                                &gl,
                                &program,
                                vb,
                                ib,
                                ic,
                                &ring_model,
                                &ring_mvp,
                                t,
                                lighten(planet.color, 0.22),
                                lighten(planet.accent, 0.12),
                                1.0,
                                0.84,
                                0.0,
                                0.0,
                                &u_mvp,
                                &u_model_view,
                                &u_time,
                                &u_color,
                                &u_accent,
                                &u_kind,
                                &u_energy,
                                &u_focus,
                                &u_gap,
                            );
                        }

                        let year_proj = project_point(
                            0.0,
                            planet.radius * 1.6,
                            0.0,
                            &multiply(proj, model_view),
                            logical_w,
                            logical_h,
                        );
                        if year_proj.visible {
                            let post_total = planet
                                .months
                                .iter()
                                .map(|slot| slot.post_indices.len())
                                .sum::<usize>();
                            next_labels.push(LabelInfo {
                                key: format!("year-{}", planet.year),
                                kind: LabelKind::Year,
                                x: year_proj.sx,
                                y: year_proj.sy,
                                opacity: year_proj.opacity,
                                width: 112.0,
                                height: 44.0,
                                anchor_left: false,
                                title: planet.year.to_string(),
                                sub: format!(
                                    "轨道 {:02} · {} 篇",
                                    planet.year - system_tick[0].year + 1,
                                    post_total
                                ),
                                posts: vec![],
                            });
                        }

                        for (slot_idx, slot) in planet.months.iter().enumerate() {
                            let label_key = format!("{}-{:02}", planet.year, slot.month);
                            let is_expanded = expanded.contains(&label_key);
                            let bob = (t * 0.9 + slot_idx as f32 * 0.72).sin() * 0.08;
                            let (mx, my, mz) = month_position(slot.angle, planet.ring_radius, bob);
                            let node_model = multiply(
                                model_view,
                                multiply(
                                    ring_rotation,
                                    multiply(
                                        translation(mx, my, mz),
                                        scale(
                                            0.19 + slot.post_indices.len() as f32 * 0.012,
                                            0.19,
                                            0.19,
                                        ),
                                    ),
                                ),
                            );
                            let node_mvp = multiply(proj, node_model);
                            if let Some((ref vb, ref ib, ic)) = sphere {
                                draw_indexed(
                                    &gl,
                                    &program,
                                    vb,
                                    ib,
                                    ic,
                                    &node_model,
                                    &node_mvp,
                                    t,
                                    lighten(planet.accent, 0.05),
                                    lighten(planet.color, 0.28),
                                    2.0,
                                    0.72,
                                    0.0,
                                    0.0,
                                    &u_mvp,
                                    &u_model_view,
                                    &u_time,
                                    &u_color,
                                    &u_accent,
                                    &u_kind,
                                    &u_energy,
                                    &u_focus,
                                    &u_gap,
                                );
                            }

                            let center_proj = project_point(
                                0.0,
                                0.0,
                                0.0,
                                &multiply(proj, model_view),
                                logical_w,
                                logical_h,
                            );
                            let label_proj = project_point(
                                mx,
                                my,
                                mz,
                                &multiply(proj, multiply(model_view, ring_rotation)),
                                logical_w,
                                logical_h,
                            );
                            if !label_proj.visible || !center_proj.visible {
                                continue;
                            }

                            let dx = label_proj.sx - center_proj.sx;
                            let dy = label_proj.sy - center_proj.sy;
                            let len = (dx * dx + dy * dy).sqrt().max(1.0);
                            let away_x = dx / len;
                            let away_y = dy / len;
                            let width = if is_expanded { 176.0 } else { 132.0 };
                            let visible_posts = slot
                                .post_indices
                                .iter()
                                .filter_map(|&idx| toc_for_tick.get(idx))
                                .map(|item| MonthPostLink {
                                    title: item.title.clone(),
                                    path: item.path.clone(),
                                })
                                .collect::<Vec<_>>();

                            next_labels.push(LabelInfo {
                                key: label_key,
                                kind: LabelKind::Month,
                                x: label_proj.sx + away_x * 42.0,
                                y: label_proj.sy + away_y * 28.0,
                                opacity: label_proj.opacity,
                                width,
                                height: if is_expanded {
                                    54.0 + visible_posts.len() as f64 * 32.0
                                } else {
                                    44.0
                                },
                                anchor_left: away_x < 0.0,
                                title: month_label(slot.month),
                                sub: format!("{} 篇", slot.post_indices.len()),
                                posts: visible_posts,
                            });
                        }
                    }

                    labels_tick.set(relax_labels(next_labels, logical_w, logical_h));
                }));
            }

            move || {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    macro_rules! rm_listener {
                        ($event:expr, $handler:expr) => {
                            if let Some(h) = $handler.as_ref() {
                                let _ = canvas.remove_event_listener_with_callback(
                                    $event,
                                    h.as_ref().unchecked_ref(),
                                );
                            }
                        };
                    }

                    rm_listener!("mousedown", on_mousedown);
                    rm_listener!("mousemove", on_mousemove);
                    rm_listener!("mouseup", on_mouseup);
                    rm_listener!("mouseleave", on_mouseleave);
                    rm_listener!("wheel", on_wheel);
                    rm_listener!("touchstart", on_touchstart);
                    rm_listener!("touchmove", on_touchmove);
                    rm_listener!("touchend", on_touchend);
                    rm_listener!("touchcancel", on_touchend);
                }
                drop(tick);
                drop(on_mousedown);
                drop(on_mousemove);
                drop(on_mouseup);
                drop(on_mouseleave);
                drop(on_wheel);
                drop(on_touchstart);
                drop(on_touchmove);
                drop(on_touchend);
            }
        });
    }

    let toggle_month = {
        let expanded_months = expanded_months.clone();
        Callback::from(move |key: String| {
            let mut next = (*expanded_months).clone();
            if next.contains(&key) {
                next.remove(&key);
            } else {
                next.insert(key);
            }
            expanded_months.set(next);
        })
    };

    let label_nodes = (*labels)
        .iter()
        .map(|label| match label.kind {
            LabelKind::Year => {
                let style = format!(
                    "left:{:.1}px;top:{:.1}px;opacity:{:.3};",
                    label.x, label.y, label.opacity
                );
                html! {
                    <div class="ring-year-label" style={style}>
                        <span class="ring-year-value">{ &label.title }</span>
                        <span class="ring-year-sub">{ &label.sub }</span>
                    </div>
                }
            }
            LabelKind::Month => {
                let is_expanded = expanded_months.contains(&label.key);
                let anchor = if label.anchor_left {
                    "ring-month-card is-left"
                } else {
                    "ring-month-card"
                };
                let card_class = if is_expanded {
                    classes!(anchor, "is-expanded")
                } else {
                    classes!(anchor, "is-collapsed")
                };
                let style = format!(
                    "left:{:.1}px;top:{:.1}px;opacity:{:.3};",
                    label.x, label.y, label.opacity
                );
                let key = label.key.clone();
                let toggle = {
                    let toggle_month = toggle_month.clone();
                    Callback::from(move |_| toggle_month.emit(key.clone()))
                };
                html! {
                    <div class={card_class} style={style}>
                        <button class="ring-month-toggle" onclick={toggle}>
                            <span class="ring-month-name">{ &label.title }</span>
                            <span class="ring-month-toggle-side">
                                <span class="ring-month-count">{ &label.sub }</span>
                                <span class="ring-month-chevron">{ if is_expanded { "−" } else { "+" } }</span>
                            </span>
                        </button>
                        <div class="ring-post-wrap">
                            <div class="ring-post-list">
                                {
                                    for label.posts.iter().map(|post| {
                                        let path = post.path.clone();
                                        let onclick = {
                                            let on_open_post = on_open_post.clone();
                                            Callback::from(move |_| on_open_post.emit(path.clone()))
                                        };
                                        html! {
                                            <button class="ring-post-pill" {onclick}>
                                                { &post.title }
                                            </button>
                                        }
                                    })
                                }
                            </div>
                        </div>
                    </div>
                }
            }
        })
        .collect::<Html>();

    html! {
        <section class="ring-scene">
            <div class="ring-view">
                <div class="ring-backdrop" />
                <div class="ring-stars ring-stars-a" />
                <div class="ring-stars ring-stars-b" />
                <canvas ref={canvas_ref} class="ring-canvas" />
                <div class="ring-label-layer">
                    { label_nodes }
                </div>
                <div class="ring-hint">{ "Drag to orbit · Scroll to zoom" }</div>
            </div>
        </section>
    }
}

fn setup_program(gl: &WebGlRenderingContext) -> Option<WebGlProgram> {
    let vert = compile_shader(
        gl,
        WebGlRenderingContext::VERTEX_SHADER,
        include_str!("shaders/ring.vert"),
    )?;
    let frag = compile_shader(
        gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        include_str!("shaders/ring.frag"),
    )?;
    link_program(gl, &vert, &frag)
}

fn compile_shader(gl: &WebGlRenderingContext, kind: u32, src: &str) -> Option<WebGlShader> {
    let shader = gl.create_shader(kind)?;
    gl.shader_source(&shader, src);
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
    vertex: &WebGlShader,
    fragment: &WebGlShader,
) -> Option<WebGlProgram> {
    let program = gl.create_program()?;
    gl.attach_shader(&program, vertex);
    gl.attach_shader(&program, fragment);
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

fn upload_indexed(
    gl: &WebGlRenderingContext,
    vertices: &[f32],
    indices: &[u16],
) -> Option<(WebGlBuffer, WebGlBuffer, i32)> {
    let vertex_buffer = gl.create_buffer()?;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &js_sys::Float32Array::from(vertices),
        WebGlRenderingContext::STATIC_DRAW,
    );

    let index_buffer = gl.create_buffer()?;
    gl.bind_buffer(
        WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
        Some(&index_buffer),
    );
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
        &js_sys::Uint16Array::from(indices),
        WebGlRenderingContext::STATIC_DRAW,
    );

    Some((vertex_buffer, index_buffer, indices.len() as i32))
}

fn make_torus_buffers(
    gl: &WebGlRenderingContext,
    major_r: f32,
    minor_r: f32,
    major_segs: u32,
    minor_segs: u32,
) -> Option<(WebGlBuffer, WebGlBuffer, i32)> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=major_segs {
        let u = i as f32 / major_segs as f32;
        let theta = u * PI * 2.0;
        let (sin_theta, cos_theta) = theta.sin_cos();
        for j in 0..=minor_segs {
            let v = j as f32 / minor_segs as f32;
            let phi = v * PI * 2.0;
            let (sin_phi, cos_phi) = phi.sin_cos();
            let ring = major_r + minor_r * cos_phi;
            let x = ring * cos_theta;
            let y = minor_r * sin_phi;
            let z = ring * sin_theta;
            let nx = cos_theta * cos_phi;
            let ny = sin_phi;
            let nz = sin_theta * cos_phi;
            vertices.extend_from_slice(&[x, y, z, nx, ny, nz, u, v]);
        }
    }

    for i in 0..major_segs {
        for j in 0..minor_segs {
            let a = (i * (minor_segs + 1) + j) as u16;
            let b = ((i + 1) * (minor_segs + 1) + j) as u16;
            let c = ((i + 1) * (minor_segs + 1) + j + 1) as u16;
            let d = (i * (minor_segs + 1) + j + 1) as u16;
            indices.extend_from_slice(&[a, b, d, b, c, d]);
        }
    }

    upload_indexed(gl, &vertices, &indices)
}

fn make_sphere_buffers(
    gl: &WebGlRenderingContext,
    radius: f32,
    lat_segs: u32,
    lon_segs: u32,
) -> Option<(WebGlBuffer, WebGlBuffer, i32)> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=lat_segs {
        let v = i as f32 / lat_segs as f32;
        let phi = v * PI;
        let (sin_phi, cos_phi) = phi.sin_cos();
        for j in 0..=lon_segs {
            let u = j as f32 / lon_segs as f32;
            let theta = u * PI * 2.0;
            let (sin_theta, cos_theta) = theta.sin_cos();
            let nx = sin_phi * cos_theta;
            let ny = cos_phi;
            let nz = sin_phi * sin_theta;
            vertices.extend_from_slice(&[radius * nx, radius * ny, radius * nz, nx, ny, nz, u, v]);
        }
    }

    for i in 0..lat_segs {
        for j in 0..lon_segs {
            let a = (i * (lon_segs + 1) + j) as u16;
            let b = ((i + 1) * (lon_segs + 1) + j) as u16;
            let c = ((i + 1) * (lon_segs + 1) + j + 1) as u16;
            let d = (i * (lon_segs + 1) + j + 1) as u16;
            indices.extend_from_slice(&[a, b, d, b, c, d]);
        }
    }

    upload_indexed(gl, &vertices, &indices)
}

#[allow(clippy::too_many_arguments)]
fn draw_indexed(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    vertex_buffer: &WebGlBuffer,
    index_buffer: &WebGlBuffer,
    index_count: i32,
    model_view: &[f32; 16],
    mvp: &[f32; 16],
    t: f32,
    color: [f32; 3],
    accent: [f32; 3],
    kind: f32,
    energy: f32,
    focus: f32,
    gap: f32,
    u_mvp: &Option<web_sys::WebGlUniformLocation>,
    u_model_view: &Option<web_sys::WebGlUniformLocation>,
    u_time: &Option<web_sys::WebGlUniformLocation>,
    u_color: &Option<web_sys::WebGlUniformLocation>,
    u_accent: &Option<web_sys::WebGlUniformLocation>,
    u_kind: &Option<web_sys::WebGlUniformLocation>,
    u_energy: &Option<web_sys::WebGlUniformLocation>,
    u_focus: &Option<web_sys::WebGlUniformLocation>,
    u_gap: &Option<web_sys::WebGlUniformLocation>,
) {
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(vertex_buffer));
    gl.bind_buffer(
        WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
        Some(index_buffer),
    );

    let pos_loc = gl.get_attrib_location(program, "a_position");
    let normal_loc = gl.get_attrib_location(program, "a_normal");
    let uv_loc = gl.get_attrib_location(program, "a_uv");
    let stride = (8 * std::mem::size_of::<f32>()) as i32;

    if pos_loc >= 0 {
        gl.vertex_attrib_pointer_with_i32(
            pos_loc as u32,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            stride,
            0,
        );
        gl.enable_vertex_attrib_array(pos_loc as u32);
    }
    if normal_loc >= 0 {
        gl.vertex_attrib_pointer_with_i32(
            normal_loc as u32,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            stride,
            (3 * std::mem::size_of::<f32>()) as i32,
        );
        gl.enable_vertex_attrib_array(normal_loc as u32);
    }
    if uv_loc >= 0 {
        gl.vertex_attrib_pointer_with_i32(
            uv_loc as u32,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            stride,
            (6 * std::mem::size_of::<f32>()) as i32,
        );
        gl.enable_vertex_attrib_array(uv_loc as u32);
    }

    if let Some(loc) = u_mvp {
        gl.uniform_matrix4fv_with_f32_array(Some(loc), false, mvp);
    }
    if let Some(loc) = u_model_view {
        gl.uniform_matrix4fv_with_f32_array(Some(loc), false, model_view);
    }
    if let Some(loc) = u_time {
        gl.uniform1f(Some(loc), t);
    }
    if let Some(loc) = u_color {
        gl.uniform3f(Some(loc), color[0], color[1], color[2]);
    }
    if let Some(loc) = u_accent {
        gl.uniform3f(Some(loc), accent[0], accent[1], accent[2]);
    }
    if let Some(loc) = u_kind {
        gl.uniform1f(Some(loc), kind);
    }
    if let Some(loc) = u_energy {
        gl.uniform1f(Some(loc), energy);
    }
    if let Some(loc) = u_focus {
        gl.uniform1f(Some(loc), focus.rem_euclid(1.0));
    }
    if let Some(loc) = u_gap {
        gl.uniform1f(Some(loc), gap.max(0.0));
    }

    gl.draw_elements_with_i32(
        WebGlRenderingContext::TRIANGLES,
        index_count,
        WebGlRenderingContext::UNSIGNED_SHORT,
        0,
    );
}

#[derive(Clone, Copy)]
struct Projection {
    sx: f64,
    sy: f64,
    opacity: f64,
    visible: bool,
}

fn project_point(x: f32, y: f32, z: f32, mvp: &[f32; 16], w: f64, h: f64) -> Projection {
    let cx = mvp[0] * x + mvp[4] * y + mvp[8] * z + mvp[12];
    let cy = mvp[1] * x + mvp[5] * y + mvp[9] * z + mvp[13];
    let cz = mvp[2] * x + mvp[6] * y + mvp[10] * z + mvp[14];
    let cw = mvp[3] * x + mvp[7] * y + mvp[11] * z + mvp[15];
    if cw.abs() < 0.001 {
        return Projection {
            sx: 0.0,
            sy: 0.0,
            opacity: 0.0,
            visible: false,
        };
    }

    let nx = cx / cw;
    let ny = cy / cw;
    let nz = cz / cw;
    let sx = (nx as f64 + 1.0) * 0.5 * w;
    let sy = (1.0 - ny as f64) * 0.5 * h;
    let visible = cw > 0.0 && nx.abs() < 1.18 && ny.abs() < 1.14 && (-1.1..=1.1).contains(&nz);
    let opacity = (1.0 - ((nz as f64 + 1.0) * 0.5)).clamp(0.28, 1.0);

    Projection {
        sx,
        sy,
        opacity,
        visible,
    }
}

fn transform_point(matrix: &[f32; 16], x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    (
        matrix[0] * x + matrix[4] * y + matrix[8] * z + matrix[12],
        matrix[1] * x + matrix[5] * y + matrix[9] * z + matrix[13],
        matrix[2] * x + matrix[6] * y + matrix[10] * z + matrix[14],
    )
}

fn relax_labels(labels: Vec<LabelInfo>, width: f64, height: f64) -> Vec<LabelInfo> {
    let mut years = Vec::new();
    let mut left = Vec::new();
    let mut right = Vec::new();

    for label in labels {
        match label.kind {
            LabelKind::Year => years.push(label),
            LabelKind::Month => {
                if label.anchor_left {
                    left.push(label);
                } else {
                    right.push(label);
                }
            }
        }
    }

    fn settle(side: &mut [LabelInfo], width: f64, height: f64) {
        side.sort_by(|a, b| a.y.total_cmp(&b.y));
        let pad = 12.0;
        let gap = 10.0;
        let mut current_y = 32.0;
        for label in side.iter_mut() {
            label.y = label.y.max(current_y + label.height * 0.5);
            label.y = label.y.min(height - label.height * 0.5 - 18.0);
            current_y = label.y + label.height * 0.5 + gap;
            if label.anchor_left {
                label.x = label.x.max(pad + label.width * 0.5);
            } else {
                label.x = label.x.min(width - pad - label.width * 0.5);
            }
        }
    }

    settle(&mut left, width, height);
    settle(&mut right, width, height);

    years.extend(left);
    years.extend(right);
    years
}

fn month_position(angle: f32, radius: f32, bob: f32) -> (f32, f32, f32) {
    let (sin_angle, cos_angle) = angle.sin_cos();
    (cos_angle * radius, bob, sin_angle * radius)
}

fn lighten(color: [f32; 3], amount: f32) -> [f32; 3] {
    [
        (color[0] + amount).min(1.0),
        (color[1] + amount).min(1.0),
        (color[2] + amount).min(1.0),
    ]
}

fn month_label(month: u32) -> String {
    format!("{month:02} 月")
}

fn multiply(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut out = [0.0_f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            out[col * 4 + row] = a[row] * b[col * 4]
                + a[4 + row] * b[col * 4 + 1]
                + a[8 + row] * b[col * 4 + 2]
                + a[12 + row] * b[col * 4 + 3];
        }
    }
    out
}

fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fovy * 0.5).tan();
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
        2.0 * far * near * nf,
        0.0,
    ]
}

fn rotation_x(rad: f32) -> [f32; 16] {
    let (sin_rad, cos_rad) = rad.sin_cos();
    [
        1.0, 0.0, 0.0, 0.0, 0.0, cos_rad, sin_rad, 0.0, 0.0, -sin_rad, cos_rad, 0.0, 0.0, 0.0, 0.0,
        1.0,
    ]
}

fn rotation_y(rad: f32) -> [f32; 16] {
    let (sin_rad, cos_rad) = rad.sin_cos();
    [
        cos_rad, 0.0, -sin_rad, 0.0, 0.0, 1.0, 0.0, 0.0, sin_rad, 0.0, cos_rad, 0.0, 0.0, 0.0, 0.0,
        1.0,
    ]
}

fn rotation_z(rad: f32) -> [f32; 16] {
    let (sin_rad, cos_rad) = rad.sin_cos();
    [
        cos_rad, sin_rad, 0.0, 0.0, -sin_rad, cos_rad, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
        1.0,
    ]
}

fn translation(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}

fn scale(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}
