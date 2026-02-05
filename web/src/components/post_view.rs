use crate::components::PostPayload;
use crate::components::{card::Card, page::Page};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PostViewProps {
    pub post: PostPayload,
    pub on_home: Callback<()>,
}

#[function_component(PostView)]
pub fn post_view(props: &PostViewProps) -> Html {
    let injected =
        yew::virtual_dom::VNode::from_html_unchecked(AttrValue::from(props.post.content.clone()));

    let on_home = {
        let cb = props.on_home.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let header = html! {
        <header class="header">
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
        <Page {header}>
            <hr class="divider" />
            <Card class={classes!("article")}>
                { injected }
            </Card>
        </Page>
    }
}
