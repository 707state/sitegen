use serde::Deserialize;
use yew::prelude::*;

use gloo_net::http::Request;
use std::collections::{HashMap, HashSet};

const BASE_CSS: &str = r#"
:root {
    --sky-900: #0b3b5a;
    --sky-700: #1b6aa5;
    --sky-500: #3aa2e3;
    --sky-300: #8fd0f3;
    --sky-100: #e7f6ff;
    --ink: #0f1f2b;
    --card: #f7fcff;
    --border: #cfe9f8;
}

* {
    box-sizing: border-box;
}

body {
    margin: 0;
    background: radial-gradient(1200px 600px at 20% 0%, #dff3ff 0%, #f7fbff 50%, #ffffff 100%);
    color: var(--ink);
    font-family: "Trebuchet MS", "Segoe UI", sans-serif;
}

.page {
    min-height: 100vh;
    padding: 32px 20px 48px;
    display: grid;
    gap: 16px;
    align-content: start;
    animation: page-fade 600ms ease-out;
}

.header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
}

.title {
    font-size: 32px;
    margin: 0;
    color: var(--sky-900);
    letter-spacing: 0.4px;
}

.subtitle {
    margin: 6px 0 0;
    color: var(--sky-700);
    font-size: 14px;
}

.card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 6px 14px;
    box-shadow: 0 10px 30px rgba(20, 90, 130, 0.08);
    animation: lift-in 500ms ease-out;
}

.topic-button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    border: none;
    background: transparent;
    color: var(--sky-900);
    font-size: 18px;
    font-weight: 700;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 10px;
    transition: background 180ms ease, color 180ms ease, transform 180ms ease;
}

.topic-button:hover {
    background: var(--sky-100);
    color: var(--sky-700);
    transform: translateY(-1px);
}

.topic-button:active {
    transform: translateY(0);
}

.list {
    list-style: none;
    padding-left: 0;
    margin: 8px 0 0;
    display: grid;
    gap: 8px;
}

.link-button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 10px;
    border: 1px solid transparent;
    background: #ffffff;
    color: var(--sky-900);
    cursor: pointer;
    transition: border 180ms ease, box-shadow 180ms ease, transform 180ms ease;
}

.link-button:hover {
    border-color: var(--border);
    box-shadow: 0 8px 16px rgba(20, 90, 130, 0.1);
    transform: translateY(-1px);
}

.home-button {
    border: none;
    background: linear-gradient(135deg, var(--sky-500), var(--sky-700));
    color: #ffffff;
    padding: 8px 14px;
    border-radius: 999px;
    cursor: pointer;
    font-weight: 600;
    transition: transform 180ms ease, box-shadow 180ms ease;
}

.home-button:hover {
    transform: translateY(-1px);
    box-shadow: 0 10px 20px rgba(58, 162, 227, 0.3);
}

.article {
    line-height: 1.65;
}

.divider {
    border: none;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--border), transparent);
    margin: 12px 0 18px;
}

@keyframes page-fade {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
}

@keyframes lift-in {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}

@media (max-width: 720px) {
    .page {
        padding: 24px 16px 36px;
    }

    .title {
        font-size: 26px;
    }
}
"#;

#[derive(Debug, Clone, Deserialize)]
struct FrontMatter {
    title: String,
    author: String,
    tags: Vec<String>,
    date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct PostPayload {
    path: String,
    modified_at_unix: Option<u64>,
    metadata: FrontMatter,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TocItem {
    title: String,
    path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct IndexPayload {
    paragraph_under_certain_topic: HashMap<String, Vec<String>>,
    table_of_content: Vec<TocItem>,
}

#[function_component(App)]
fn app() -> Html {
    let index = use_state(|| None::<IndexPayload>);
    let post = use_state(|| None::<PostPayload>);
    let error = use_state(|| None::<String>);
    let is_loading = use_state(|| false);
    let expanded_topics = use_state(HashSet::<String>::new);

    {
        let index = index.clone();
        let error = error.clone();
        let is_loading = is_loading.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                is_loading.set(true);
                let res = Request::get("index.json").send().await;

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

    if let Some(err) = (*error).clone() {
        return html! {
            <main class="page">
                <style>{BASE_CSS}</style>
                <header class="header">
                    <h1 class="title">{"Error"}</h1>
                </header>
                <section class="card">
                    <pre>{err}</pre>
                </section>
            </main>
        };
    }

    if *is_loading {
        return html! {
            <main class="page">
                <style>{BASE_CSS}</style>
                <header class="header">
                    <h1 class="title">{"Loading..."}</h1>
                </header>
                <section class="card">
                    <p>{"Fetching index.json"}</p>
                </section>
            </main>
        };
    };

    if let Some(p) = (*post).clone() {
        let injected = yew::virtual_dom::VNode::from_html_unchecked(AttrValue::from(p.content));
        let on_home = {
            let post = post.clone();
            Callback::from(move |_| post.set(None))
        };
        return html! {
            <main class="page">
                <style>{BASE_CSS}</style>
                <header class="header">
                    <div>
                        <h1 class="title">{ p.metadata.title }</h1>
                        <p class="subtitle">
                            {format!("{}{}", p.metadata.author, p.metadata.date.as_deref().map_or(String::new(), |d| format!(" · {d}")))}
                        </p>
                    </div>
                    <button onclick={on_home} class="home-button">{"Home"}</button>
                </header>
                <hr class="divider" />
                <article class="card article">
                    { injected }
                </article>
            </main>
        };
    }

    let Some(index_payload) = (*index).clone() else {
        return html! {
            <main class="page">
                <style>{BASE_CSS}</style>
                <header class="header">
                    <h1 class="title">{"Loading..."}</h1>
                </header>
                <section class="card">
                    <p>{"No index data yet"}</p>
                </section>
            </main>
        };
    };

    let mut title_to_path: HashMap<String, String> = HashMap::new();
    for item in &index_payload.table_of_content {
        title_to_path.insert(item.title.clone(), item.path.clone());
    }

    let mut topics: Vec<_> = index_payload.paragraph_under_certain_topic.iter().collect();
    topics.sort_by(|a, b| a.0.cmp(b.0));

    let on_open = {
        let post = post.clone();
        let error = error.clone();
        let is_loading = is_loading.clone();
        Callback::from(move |path: String| {
            let post = post.clone();
            let error = error.clone();
            let is_loading = is_loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                is_loading.set(true);
                let res = Request::get(&path).send().await;
                match res {
                    Ok(resp) => match resp.json::<PostPayload>().await {
                        Ok(p) => post.set(Some(p)),
                        Err(e) => error.set(Some(format!("JSON parse error: {e}"))),
                    },
                    Err(e) => {
                        error.set(Some(format!("Fetch error: {e}")));
                    }
                }
                is_loading.set(false);
            });
        })
    };

    html! {
        <main class="page">
            <style>{BASE_CSS}</style>
            <header class="header">
                <div>
                    <h1 class="title">{"Home"}</h1>
                    <p class="subtitle">{"Browse by topic"}</p>
                </div>
            </header>
            {
                for topics.into_iter().map(|(topic, titles)| {
                    let topic = topic.clone();
                    let titles = titles.clone();
                    let is_open = expanded_topics.contains(&topic);
                    let on_toggle = {
                        let expanded_topics = expanded_topics.clone();
                        let topic = topic.clone();
                        Callback::from(move |_| {
                            let mut next = (*expanded_topics).clone();
                            if next.contains(&topic) {
                                next.remove(&topic);
                            } else {
                                next.insert(topic.clone());
                            }
                            expanded_topics.set(next);
                        })
                    };
                    html! {
                        <section class="card">
                            <button onclick={on_toggle} class="topic-button">
                                { if is_open { "▼ " } else { "▶ " } }
                                {topic.clone()}
                            </button>
                            {
                                if is_open {
                                    html! {
                                        <ul class="list">
                                            {
                                                for titles.into_iter().map(|title| {
                                                    let path = title_to_path.get(&title).cloned();
                                                    let on_open = on_open.clone();
                                                    let onclick = match path {
                                                        Some(path) => {
                                                            let path = path.clone();
                                                            Some(Callback::from(move |_| on_open.emit(path.clone())))
                                                        }
                                                        None => None,
                                                    };
                                                    html! {
                                                        <li>
                                                            {
                                                                if let Some(onclick) = onclick {
                                                                    html! { <button onclick={onclick} class="link-button">{title}</button> }
                                                                } else {
                                                                    html! { <span>{title}</span> }
                                                                }
                                                            }
                                                        </li>
                                                    }
                                                })
                                            }
                                        </ul>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </section>
                    }
                })
            }
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
