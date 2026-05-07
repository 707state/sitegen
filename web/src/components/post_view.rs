use crate::components::PostPayload;
use crate::components::{card::Card, draggable_toc::DraggableToc, page::Page};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = typesetMath, catch)]
    fn typeset_math(element: &web_sys::Element) -> Result<(), JsValue>;
}

#[derive(Clone, PartialEq)]
pub struct SeriesNavItem {
    pub title: String,
    pub path: String,
}

#[derive(Properties, PartialEq)]
pub struct PostViewProps {
    pub post: PostPayload,
    pub on_home: Callback<()>,
    pub on_open_post: Callback<String>,
    pub previous_in_series: Option<SeriesNavItem>,
    pub next_in_series: Option<SeriesNavItem>,
}

#[function_component(PostView)]
pub fn post_view(props: &PostViewProps) -> Html {
    let injected =
        yew::virtual_dom::VNode::from_html_unchecked(AttrValue::from(props.post.content.clone()));
    let article_ref = use_node_ref();

    {
        let article_ref = article_ref.clone();
        let content = props.post.content.clone();
        let math_enabled = props.post.math;
        use_effect_with(content, move |_| {
            if math_enabled {
                if let Some(article) = article_ref.cast::<web_sys::Element>() {
                    let _ = typeset_math(&article);
                }
            }

            || ()
        });
    }

    let on_home = {
        let cb = props.on_home.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let series_name = props.post.metadata.series.clone();

    let header = html! {
        <header class="header post-header">
            <div>
                <h1 class="title">{ props.post.metadata.title.clone() }</h1>
                <p class="subtitle">
                    {
                        format!(
                            "{}{}",
                            props.post.metadata.author,
                            props.post.metadata.date.as_deref().map_or(String::new(), |d| format!(" · {d}"))
                        )
                    }
                </p>
            </div>
            <button onclick={on_home} class="home-button">{ "Home" }</button>
        </header>
    };

    html! {
        <Page {header} class={classes!("post-page")}>
            <hr class="divider" />
            <DraggableToc headings={props.post.headings.clone()} />
            <Card class={classes!("article")}>
                <div ref={article_ref} class="article-content">
                    { injected }
                </div>
            </Card>
            {
                if series_name.is_some()
                    && (props.previous_in_series.is_some() || props.next_in_series.is_some())
                {
                    let prev = props.previous_in_series.clone();
                    let next = props.next_in_series.clone();
                    let on_open_post = props.on_open_post.clone();
                    html! {
                        <Card class={classes!("series-nav-card")}>
                            <div class="series-nav-header">
                                <div class="series-nav-kicker">{ "Series" }</div>
                                <div class="series-nav-name">{ series_name.unwrap_or_default() }</div>
                            </div>
                            <div class="series-nav-grid">
                                {
                                    if let Some(item) = prev {
                                        let path = item.path.clone();
                                        let onclick = {
                                            let cb = on_open_post.clone();
                                            Callback::from(move |_| cb.emit(path.clone()))
                                        };
                                        html! {
                                            <button type="button" class="series-nav-button" {onclick}>
                                                <span class="series-nav-label">{ "上一篇" }</span>
                                                <span class="series-nav-title">{ item.title }</span>
                                            </button>
                                        }
                                    } else {
                                        Html::default()
                                    }
                                }
                                {
                                    if let Some(item) = next {
                                        let path = item.path.clone();
                                        let onclick = {
                                            let cb = on_open_post.clone();
                                            Callback::from(move |_| cb.emit(path.clone()))
                                        };
                                        html! {
                                            <button type="button" class="series-nav-button" {onclick}>
                                                <span class="series-nav-label">{ "下一篇" }</span>
                                                <span class="series-nav-title">{ item.title }</span>
                                            </button>
                                        }
                                    } else {
                                        Html::default()
                                    }
                                }
                            </div>
                        </Card>
                    }
                } else {
                    Html::default()
                }
            }
        </Page>
    }
}
