use crate::components::{
    PostPayload, TocItem, about_me_view::AboutMeView, error_view::ErrorView, home_view::HomeView,
    loading_view::LoadingView, post_view::PostView, post_view::SeriesNavItem,
    search_view::SearchView, webgl_cube::WebglCube,
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
    pub paragraph_under_certain_series: HashMap<String, Vec<String>>,
    pub table_of_content: Vec<TocItem>,
}

#[derive(Clone, Routable, PartialEq, Eq, Debug)]
enum Route {
    #[at("/")]
    Home,
    #[at("/*path")]
    Post { path: String },
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <HashRouter>
            <AppShell />
        </HashRouter>
    }
}

#[function_component(AppShell)]
fn app_shell() -> Html {
    let index = use_state(|| None::<IndexPayload>);
    let post = use_state(|| None::<PostPayload>);
    let error = use_state(|| None::<String>);
    let is_loading = use_state(|| false);
    let expanded_topics = use_state(HashSet::<String>::new);
    let expanded_series = use_state(HashSet::<String>::new);
    let search_keyword = use_state(String::new);
    let is_search_open = use_state(|| false);
    let is_about_open = use_state(|| false);
    let is_cube_open = use_state(|| false);
    let navigator = use_navigator();
    let route = use_route::<Route>().unwrap_or(Route::Home);
    let route_path = match &route {
        Route::Post { path } => Some(path.clone()),
        _ => None,
    };

    {
        let index = index.clone();
        let error = error.clone();
        let is_loading = is_loading.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                is_loading.set(true);
                let res = Request::get(&content_url("/index.json")).send().await;

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
    {
        let post = post.clone();
        let error = error.clone();
        let is_loading = is_loading.clone();
        use_effect_with(route_path.clone(), move |path| {
            if path.is_none() {
                post.set(None);
                error.set(None);
            } else {
                let req_path = format!(
                    "/{}",
                    path.clone().unwrap_or_default().trim_start_matches('/')
                );
                wasm_bindgen_futures::spawn_local(async move {
                    is_loading.set(true);
                    error.set(None);
                    let res = Request::get(&content_url(&req_path)).send().await;
                    match res {
                        Ok(resp) => match resp.json::<PostPayload>().await {
                            Ok(p) => post.set(Some(p)),
                            Err(e) => error.set(Some(format!("JSON parse error (post): {e}"))),
                        },
                        Err(e) => error.set(Some(format!("Fetch error (post): {e}"))),
                    }
                    is_loading.set(false);
                });
            }

            || ()
        });
    }

    let on_home = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            if let Some(nav) = navigator.clone() {
                nav.push(&Route::Home);
            }
        })
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
        let navigator = navigator.clone();
        Callback::from(move |path: String| {
            if let Some(nav) = navigator.clone() {
                nav.push(&Route::Post {
                    path: path.trim_start_matches('/').to_string(),
                });
            }
        })
    };
    let on_search = {
        let search_keyword = search_keyword.clone();
        Callback::from(move |keyword: String| {
            search_keyword.set(keyword.trim().to_string());
        })
    };
    let on_toggle_series = {
        let expanded_series = expanded_series.clone();
        Callback::from(move |series: String| {
            let mut next = (*expanded_series).clone();
            if next.contains(&series) {
                next.remove(&series);
            } else {
                next.insert(series);
            }
            expanded_series.set(next);
        })
    };
    let on_toggle_search_panel = {
        let is_search_open = is_search_open.clone();
        Callback::from(move |_| is_search_open.set(!*is_search_open))
    };
    let on_toggle_about_panel = {
        let is_about_open = is_about_open.clone();
        Callback::from(move |_| is_about_open.set(!*is_about_open))
    };
    let on_toggle_cube = {
        let is_cube_open = is_cube_open.clone();
        Callback::from(move |_| is_cube_open.set(!*is_cube_open))
    };

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
    match route {
        Route::Home => {
            let Some(index_payload) = (*index).clone() else {
                return html! {
                    <LoadingView text={"No index data yet"} />
                };
            };
            let mut topics: Vec<(String, Vec<String>)> = index_payload
                .paragraph_under_certain_topic
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            topics.sort_by(|a, b| a.0.cmp(&b.0));
            let mut series: Vec<(String, Vec<String>)> = index_payload
                .paragraph_under_certain_series
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            series.sort_by(|a, b| a.0.cmp(&b.0));
            let toc_items = index_payload.table_of_content;
            let title_to_path: HashMap<String, String> = toc_items
                .iter()
                .map(|item| (item.title.clone(), item.path.clone()))
                .collect();
            let expanded = (*expanded_topics).clone();
            let expanded_series = (*expanded_series).clone();
            let search_keyword = if (*search_keyword).trim().is_empty() {
                None
            } else {
                Some((*search_keyword).clone())
            };
            let both_collapsed = !*is_search_open && !*is_about_open;
            let content_style = if both_collapsed {
                "--content-right-pad: 140px;"
            } else {
                "--content-right-pad: 340px;"
            };
            html! {
                <>
                    <div class="home-layout" style={content_style}>
                        <HomeView
                            toc_items={toc_items.clone()}
                            topics={topics}
                            series={series}
                            title_to_path={title_to_path}
                            expanded_topics={expanded}
                            expanded_series={expanded_series}
                            on_toggle_topic={on_toggle_topic}
                            on_toggle_series={on_toggle_series}
                            on_open_post={on_open_post.clone()}
                        />
                        <SearchView
                            toc_items={toc_items}
                            keyword={search_keyword}
                            is_open={*is_search_open}
                            about_is_open={*is_about_open}
                            on_toggle_panel={on_toggle_search_panel}
                            on_search={on_search}
                            on_open_post={on_open_post}
                        />
                    </div>
                    <AboutMeView
                        nickname={"707state"}
                        avatar_url={"https://avatars.githubusercontent.com/u/115874695?v=4&size=64"}
                        bio={"Got me breathing with dragons"}
                        github_url={"https://github.com/707state"}
                        wechat_id={"visi0nist"}
                        is_open={*is_about_open}
                        on_toggle_panel={on_toggle_about_panel}
                    />
                    <WebglCube collapsed={!*is_cube_open} on_toggle={on_toggle_cube} />
                </>
            }
        }
        Route::Post { .. } => {
            if let Some(p) = (*post).clone() {
                let (previous_in_series, next_in_series) =
                    if let Some(index_payload) = (*index).clone() {
                        let title_to_path: HashMap<String, String> = index_payload
                            .table_of_content
                            .iter()
                            .map(|item| (item.title.clone(), item.path.clone()))
                            .collect();
                        series_neighbors(&p, &index_payload, &title_to_path)
                    } else {
                        (None, None)
                    };
                html! {
                    <PostView
                        post={p}
                        on_home={on_home.clone()}
                        on_open_post={on_open_post.clone()}
                        previous_in_series={previous_in_series}
                        next_in_series={next_in_series}
                    />
                }
            } else {
                html! {
                    <LoadingView text={"Loading post..."} />
                }
            }
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

fn content_url(path: &str) -> String {
    let base = option_env!("CONTENT_BASE_URL").unwrap_or("");
    let base = base.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    if base.is_empty() {
        format!("/{}", path)
    } else {
        format!("{}/{}", base, path)
    }
}

fn series_neighbors(
    post: &PostPayload,
    index_payload: &IndexPayload,
    title_to_path: &HashMap<String, String>,
) -> (Option<SeriesNavItem>, Option<SeriesNavItem>) {
    let Some(series_name) = post.metadata.series.as_ref() else {
        return (None, None);
    };
    let Some(series_titles) = index_payload
        .paragraph_under_certain_series
        .get(series_name)
    else {
        return (None, None);
    };
    let Some(current_idx) = series_titles
        .iter()
        .position(|title| title == &post.metadata.title)
    else {
        return (None, None);
    };

    let previous = if current_idx > 0 {
        nav_item_for_title(&series_titles[current_idx - 1], title_to_path)
    } else {
        None
    };
    let next = series_titles
        .get(current_idx + 1)
        .and_then(|title| nav_item_for_title(title, title_to_path));

    (previous, next)
}

fn nav_item_for_title(
    title: &str,
    title_to_path: &HashMap<String, String>,
) -> Option<SeriesNavItem> {
    title_to_path.get(title).cloned().map(|path| SeriesNavItem {
        title: title.to_string(),
        path,
    })
}
