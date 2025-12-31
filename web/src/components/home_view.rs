use crate::components::{page::Page, topic_card::TopicCard};
use std::collections::{HashMap, HashSet};
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct HomeViewProps {
    pub topics: Vec<(String, Vec<String>)>,
    pub title_to_path: HashMap<String, String>,
    pub expanded_topics: HashSet<String>,

    pub on_toggle_topic: Callback<String>,
    pub on_open_post: Callback<String>,
}

#[function_component(HomeView)]
pub fn home_view(
    HomeViewProps {
        topics,
        title_to_path,
        expanded_topics,
        on_toggle_topic,
        on_open_post,
    }: &HomeViewProps,
) -> Html {
    let header = html! {
        <header class="header">
        <div>
            <h1 class="title">{ "Home" }</h1>
            <p class="subtitle">{ "Browse by topic" }</p>
        </div>
        </header>
    };
    html! {
        <Page {header}>
            {
                for topics.iter().map(|(topic,titles)|{
                    let is_open=expanded_topics.contains(topic);
                    html! {
                        <TopicCard
                            topic={topic.clone()}
                            titles={titles.clone()}
                            title_to_path={title_to_path.clone()}
                            is_open={is_open}
                            on_toggle={on_toggle_topic.clone()}
                            on_open_post={on_open_post.clone()}
                            />
                        }
                })
            }
        </Page>
    }
}
