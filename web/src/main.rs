use serde::Deserialize;
use yew::prelude::*;

use gloo_net::http::Request;

#[derive(Debug, Clone, Deserialize)]
struct PostPayload {
    title: String,
    html: String,
}

#[function_component(App)]
fn app() -> Html {
    let payload = use_state(|| None::<PostPayload>);
    let error = use_state(|| None::<String>);

    {
        let payload = payload.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let res = Request::get("/content/hello.json").send().await;

                match res {
                    Ok(resp) => match resp.json::<PostPayload>().await {
                        Ok(p) => payload.set(Some(p)),
                        Err(e) => error.set(Some(format!("JSON parse error: {e}"))),
                    },
                    Err(e) => {
                        error.set(Some(format!("Fetch error: {e}")));
                    }
                }
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

    let Some(p) = (*payload).clone() else {
        return html! {
            <main style="font-family: sans-serif; padding: 16px;">
                <h1>{"Loading..."}</h1>
                <p>{"Fetching /content/hello.json"}</p>
            </main>
        };
    };

    // 关键：将 HTML 字符串注入为 VNode（unchecked）
    let injected = yew::virtual_dom::VNode::from_html_unchecked(AttrValue::from(p.html));

    html! {
        <main style="font-family: sans-serif; padding: 16px;">
            <h1>{ p.title }</h1>
            <hr />
            <article>
                { injected }
            </article>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
