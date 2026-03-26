use crate::components::PostHeading;
use gloo::events::EventListener;
use gloo::timers::callback::Timeout;
use gloo::utils::{document, window};
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct DragState {
    pointer_x: f64,
    pointer_y: f64,
    panel_x: f64,
    panel_y: f64,
}

#[derive(Properties, PartialEq)]
pub struct DraggableTocProps {
    pub headings: Vec<PostHeading>,
}

#[function_component(DraggableToc)]
pub fn draggable_toc(props: &DraggableTocProps) -> Html {
    if props.headings.is_empty() {
        return Html::default();
    }

    let panel_ref = use_node_ref();
    let collapsed = use_state(|| false);
    let body_hidden = use_state(|| false);
    let position = use_state(default_position);
    let drag_state = use_state(|| None::<DragState>);
    let toggle_timer = use_mut_ref(|| None::<Timeout>);

    {
        let panel_ref = panel_ref.clone();
        let position = position.clone();
        let drag_state = drag_state.clone();

        use_effect_with((*drag_state).clone(), move |active_drag| {
            let listeners = active_drag.clone().map(|active_drag| {
                let panel_ref_for_move = panel_ref.clone();
                let position_for_move = position.clone();
                let drag_state_for_up = drag_state.clone();
                let move_listener = EventListener::new(&document(), "mousemove", move |event| {
                    let Some(mouse_event) = event.dyn_ref::<web_sys::MouseEvent>() else {
                        return;
                    };
                    let (panel_w, panel_h) = panel_size(&panel_ref_for_move);
                    let next_x = active_drag.panel_x + f64::from(mouse_event.client_x())
                        - active_drag.pointer_x;
                    let next_y = active_drag.panel_y + f64::from(mouse_event.client_y())
                        - active_drag.pointer_y;
                    position_for_move.set(clamp_position(next_x, next_y, panel_w, panel_h));
                });
                let up_listener = EventListener::new(&document(), "mouseup", move |_| {
                    drag_state_for_up.set(None);
                });
                (move_listener, up_listener)
            });

            move || {
                drop(listeners);
            }
        });
    }

    let on_drag_start = {
        let drag_state = drag_state.clone();
        let position = position.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            drag_state.set(Some(DragState {
                pointer_x: f64::from(event.client_x()),
                pointer_y: f64::from(event.client_y()),
                panel_x: (*position).0,
                panel_y: (*position).1,
            }));
        })
    };

    let on_toggle = {
        let collapsed = collapsed.clone();
        let body_hidden = body_hidden.clone();
        let toggle_timer = toggle_timer.clone();
        Callback::from(move |_| {
            if let Some(timer) = toggle_timer.borrow_mut().take() {
                timer.cancel();
            }

            if *collapsed {
                collapsed.set(false);
                let body_hidden = body_hidden.clone();
                let toggle_timer_for_cleanup = toggle_timer.clone();
                let timer = Timeout::new(150, move || {
                    body_hidden.set(false);
                    toggle_timer_for_cleanup.borrow_mut().take();
                });
                *toggle_timer.borrow_mut() = Some(timer);
            } else {
                body_hidden.set(true);
                let collapsed = collapsed.clone();
                let toggle_timer_for_cleanup = toggle_timer.clone();
                let timer = Timeout::new(210, move || {
                    collapsed.set(true);
                    toggle_timer_for_cleanup.borrow_mut().take();
                });
                *toggle_timer.borrow_mut() = Some(timer);
            }
        })
    };

    let panel_style = format!("left: {:.0}px; top: {:.0}px;", (*position).0, (*position).1);
    let panel_classes = classes!(
        "post-toc-panel",
        (*collapsed).then_some("is-collapsed"),
        (*body_hidden).then_some("body-hidden"),
        (*drag_state).is_some().then_some("is-dragging")
    );

    html! {
        <aside ref={panel_ref} class={panel_classes} style={panel_style}>
            <div class="post-toc-card">
                <div class="post-toc-toolbar">
                    <button type="button" class="post-toc-drag-handle" onmousedown={on_drag_start}>
                        { "目录" }
                    </button>
                    <button type="button" class="post-toc-toggle" onclick={on_toggle}>
                        { if *collapsed { "展开" } else { "收起" } }
                    </button>
                </div>
                <div class={classes!("post-toc-body", (*body_hidden).then_some("is-collapsed"))}>
                    <div class="post-toc-title">{ "目录" }</div>
                    <div class="post-toc-list">
                        { for props.headings.iter().map(render_heading) }
                    </div>
                </div>
            </div>
        </aside>
    }
}

fn render_heading(heading: &PostHeading) -> Html {
    let heading_id = heading.id.clone();
    let onclick = Callback::from(move |_| {
        if let Some(target) = document().get_element_by_id(&heading_id) {
            target.scroll_into_view();
        }
    });

    let style = format!("--toc-level: {};", heading.level.saturating_sub(1));

    html! {
        <button type="button" class="post-toc-link" {onclick} style={style}>
            { heading.text.clone() }
        </button>
    }
}

fn default_position() -> (f64, f64) {
    let viewport_w = window_size().0;
    ((viewport_w - 336.0).max(12.0), 132.0)
}

fn panel_size(panel_ref: &NodeRef) -> (f64, f64) {
    if let Some(element) = panel_ref.cast::<web_sys::Element>() {
        let rect = element.get_bounding_client_rect();
        let width = rect.width().max(72.0);
        let height = rect.height().max(72.0);
        (width, height)
    } else {
        (280.0, 240.0)
    }
}

fn clamp_position(x: f64, y: f64, panel_w: f64, panel_h: f64) -> (f64, f64) {
    let (viewport_w, viewport_h) = window_size();
    let x = x.max(12.0).min((viewport_w - panel_w - 12.0).max(12.0));
    let y = y.max(12.0).min((viewport_h - panel_h - 12.0).max(12.0));
    (x, y)
}

fn window_size() -> (f64, f64) {
    let viewport_w = window()
        .inner_width()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(1280.0);
    let viewport_h = window()
        .inner_height()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(720.0);
    (viewport_w, viewport_h)
}
