use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct LoadingViewProps {
    #[prop_or_else(|| "Loading...".into())]
    pub title: AttrValue,

    #[prop_or_default]
    pub text: Option<AttrValue>,
}
#[function_component(LoadingView)]
pub fn loading_view(props: &LoadingViewProps) -> Html {
    html! {
        <main class="page">
            <header class="header">
                <h1 class="title">{ props.title.clone() }</h1>
            </header>

            <section class="card">
                {
                    if let Some(text) = props.text.clone() {
                        html! { <p>{ text }</p> }
                    } else {
                        html! { <p>{ "Please wait…" }</p> }
                    }
                }
            </section>
        </main>
    }
}
