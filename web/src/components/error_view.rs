use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorViewProps {
    pub message: String,

    #[prop_or_default]
    pub on_home: Option<Callback<()>>,
}

#[function_component(ErrorView)]
pub fn error_view(props: &ErrorViewProps) -> Html {
    let on_home_click = props.on_home.as_ref().map(|cb| {
        let cb = cb.clone();
        Callback::from(move |_| cb.emit(()))
    });

    html! {
        <main class="page">
            <header class="header">
                <div>
                    <h1 class="title">{ "Error" }</h1>
                    <p class="subtitle">{ "Something went wrong" }</p>
                </div>

                {
                    if let Some(onclick) = on_home_click {
                        html! { <button onclick={onclick} class="home-button">{ "Home" }</button> }
                    } else {
                        html! {}
                    }
                }
            </header>

            <section class="card">
                <pre style="white-space: pre-wrap; margin: 0;">
                    { props.message.clone() }
                </pre>
            </section>
        </main>
    }
}
