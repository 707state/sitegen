use serde::Deserialize;
use yew::prelude::*;

use gloo_net::http::Request;
use std::collections::HashMap;

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
            <main style="font-family: sans-serif; padding: 16px;">
                <h1>{"Error"}</h1>
                <pre>{err}</pre>
            </main>
        };
    }

    if *is_loading {
        return html! {
            <main style="font-family: sans-serif; padding: 16px;">
                <h1>{"Loading..."}</h1>
                <p>{"Fetching index.json"}</p>
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
            <main style="font-family: sans-serif; padding: 16px;">
                <button onclick={on_home} style="margin-bottom: 12px;">{"Home"}</button>
                <h1>{ p.metadata.title }</h1>
                <hr />
                <article>
                    { injected }
                </article>
            </main>
        };
    }

    let Some(index_payload) = (*index).clone() else {
        return html! {
            <main style="font-family: sans-serif; padding: 16px;">
                <h1>{"Loading..."}</h1>
                <p>{"No index data yet"}</p>
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
        <main style="font-family: sans-serif; padding: 16px;">
            <h1>{"Home"}</h1>
            {
                for topics.into_iter().map(|(topic, titles)| {
                    let topic = topic.clone();
                    let titles = titles.clone();
                    html! {
                        <section style="margin-bottom: 16px;">
                            <h2>{topic}</h2>
                            <ul>
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
                                                        html! { <button onclick={onclick}>{title}</button> }
                                                    } else {
                                                        html! { <span>{title}</span> }
                                                    }
                                                }
                                            </li>
                                        }
                                    })
                                }
                            </ul>
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
