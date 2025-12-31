use crate::components::card::Card;
use std::collections::HashMap;
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct TopicCardProps {
    pub topic: String,
    pub titles: Vec<String>,
    pub title_to_path: HashMap<String, String>,
    pub is_open: bool,

    pub on_toggle: Callback<String>,
    pub on_open_post: Callback<String>,
}

#[function_component(TopicCard)]
pub fn topic_card(
    TopicCardProps {
        topic,
        titles,
        title_to_path,
        is_open,
        on_toggle,
        on_open_post,
    }: &TopicCardProps,
) -> Html {
    let topic = topic.clone();
    let on_toggle_cb = {
        let topic = topic.clone();
        let cb = on_toggle.clone();
        Callback::from(move |_| cb.emit(topic.clone()))
    };
    html! {
        <Card>
        <button onclick={on_toggle_cb} class="topic-button">
            { if *is_open { "▼ " } else { "▶ " } }
            { &topic }
        </button>
        {
            if *is_open{
                html! {
                    <ul class="list">
                        { for titles.iter().map(|title| {
                            let path = title_to_path.get(title).cloned();
                            if let Some(path) = path {
                                let on_open = {
                                    let path = path.clone();
                                    let cb = on_open_post.clone();
                                    Callback::from(move |_| cb.emit(path.clone()))
                                };
                                html! {
                                    <li>
                                        <button onclick={on_open} class="link-button">
                                            { title }
                                        </button>
                                    </li>
                                }
                            } else {
                                html! { <li><span>{ title }</span></li> }
                            }
                        })}
                    </ul>
                }
            }else{
                html!{}
            }
        }
        </Card>
    }
}
