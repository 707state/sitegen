use crate::components::{
    PostPayload, TocItem, error_view::ErrorView, home_view::HomeView, loading_view::LoadingView,
    post_view::PostView, search_view::SearchView,
};
use gloo_net::http::Request;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use yew::prelude::*;
use yew_router::prelude::*;

pub mod components;

#[derive(Debug, Clone, Deserialize)]
pub struct IndexPayload {
    pub paragraph_under_certain_topic: HashMap<String, Vec<String>>,
    pub table_of_content: Vec<TocItem>,
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/search")]
    SearchHome,
    #[at("/search/:keyword")]
    Search { keyword: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(App)]
fn app() -> Html {
    let index = use_state(|| None::<IndexPayload>);
    let post = use_state(|| None::<PostPayload>);
    let error = use_state(|| None::<String>);
    let is_loading = use_state(|| false);
    let expanded_topics = use_state(HashSet::<String>::new);
    let route = use_route::<Route>();
    let navigator = use_navigator();

    {
        let index = index.clone();
        let error = error.clone();
        let is_loading = is_loading.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                is_loading.set(true);
                let res = Request::get("/index.json").send().await;

                match res {
                    Ok(resp) => match resp.json::<IndexPayload>().await {
                        Ok(p) => index.set(Some(p)),
                        Err(e) => error.set(Some(format!("JSON parse error: {e}"))),
                    },
                    Err(e) => {
                        error.set(Some(format!("Fetch error: {e}")));
                    }
                }
                is_loading.set(false);
            });

            || ()
        });
    }
    let on_home = {
        let post = post.clone();
        Callback::from(move |_| post.set(None))
    };

    let on_toggle_topic = {
        let expanded_topics = expanded_topics.clone();
        Callback::from(move |topic: String| {
            let mut next = (*expanded_topics).clone();
            if next.contains(&topic) {
                next.remove(&topic);
            } else {
                next.insert(topic);
            }
            expanded_topics.set(next);
        })
    };
    let on_open_post = {
        let post = post.clone();
        let error = error.clone();
        let is_loading = is_loading.clone();

        Callback::from(move |path: String| {
            let post = post.clone();
            let error = error.clone();
            let is_loading = is_loading.clone();

            wasm_bindgen_futures::spawn_local(async move {
                is_loading.set(true);
                error.set(None);

                let req_path = format!("/{}", path.trim_start_matches('/'));
                let res = Request::get(&req_path).send().await;
                match res {
                    Ok(resp) => match resp.json::<PostPayload>().await {
                        Ok(p) => post.set(Some(p)),
                        Err(e) => error.set(Some(format!("JSON parse error (post): {e}"))),
                    },
                    Err(e) => error.set(Some(format!("Fetch error (post): {e}"))),
                }

                is_loading.set(false);
            });
        })
    };
    let on_search = {
        let navigator = navigator.clone();
        Callback::from(move |keyword: String| {
            if let Some(navigator) = navigator.clone() {
                let trimmed = keyword.trim().to_string();
                if trimmed.is_empty() {
                    navigator.push(&Route::SearchHome);
                } else {
                    navigator.push(&Route::Search { keyword: trimmed });
                }
            }
        })
    };
    if matches!(route.as_ref(), Some(Route::NotFound)) {
        return html! {
            <ErrorView message={"Page not found".to_string()} on_home={on_home.clone()} />
        };
    }

    if let Some(err) = (*error).clone() {
        return html! {
            <ErrorView message={err} on_home={on_home.clone()} />
        };
    }

    if *is_loading {
        return html! {
            <LoadingView text={"Loading..."} />
        };
    }
    if let Some(p) = (*post).clone() {
        return html! {
            <PostView post={p} on_home={on_home.clone()} />
        };
    }
    let Some(index_payload) = (*index).clone() else {
        return html! {
            <LoadingView text={"No index data yet"} />
        };
    };
    let mut map: HashMap<String, String> = HashMap::new();
    for item in &index_payload.table_of_content {
        map.insert(item.title.clone(), item.path.clone());
    }
    let title_to_path: HashMap<String, String> = map.clone();

    // sort topics by name
    let mut topics: Vec<(String, Vec<String>)> = index_payload
        .paragraph_under_certain_topic
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    topics.sort_by(|a, b| a.0.cmp(&b.0));
    let toc_items = index_payload.table_of_content;
    let expanded = (*expanded_topics).clone();
    let search_keyword = match route.as_ref() {
        Some(Route::Search { keyword }) => Some(keyword.clone()),
        _ => None,
    };
    html! {
        <div class="home-layout">
            <HomeView
                toc_items={toc_items.clone()}
                topics={topics}
                title_to_path={title_to_path}
                expanded_topics={expanded}
                on_toggle_topic={on_toggle_topic}
                on_open_post={on_open_post.clone()}
            />
            <SearchView
                toc_items={toc_items}
                keyword={search_keyword}
                on_search={on_search}
                on_open_post={on_open_post}
            />
        </div>
    }
}

fn main() {
    yew::Renderer::<Root>::new().render();
}

#[function_component(Root)]
fn root() -> Html {
    html! {
        <BrowserRouter>
            <App />
        </BrowserRouter>
    }
}
