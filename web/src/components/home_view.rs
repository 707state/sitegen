use crate::components::{TocItem, archive_view::ArchiveView, page::Page, topic_card::TopicCard};
use std::collections::{HashMap, HashSet};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HomeViewProps {
    pub toc_items: Vec<TocItem>,
    pub topics: Vec<(String, Vec<String>)>,
    pub title_to_path: HashMap<String, String>,
    pub expanded_topics: HashSet<String>,

    pub on_toggle_topic: Callback<String>,
    pub on_open_post: Callback<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HomeMode {
    Archive,
    Topics,
}
#[function_component(HomeView)]
pub fn home_view(
    HomeViewProps {
        toc_items,
        topics,
        title_to_path,
        expanded_topics,
        on_toggle_topic,
        on_open_post,
    }: &HomeViewProps,
) -> Html {
    let mode = use_state(|| HomeMode::Archive);
    let set_archive = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(HomeMode::Archive))
    };
    let set_topics = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(HomeMode::Topics))
    };
    let header = {
        let is_archive = *mode == HomeMode::Archive;
        let is_topics = *mode == HomeMode::Topics;

        html! {
            <header class="header">
                <div>
                    <h1 class="title">{ "Home" }</h1>
                    <p class="subtitle">
                        {
                            if is_archive { "Archive (by date)" }
                            else { "Browse by topic" }
                        }
                    </p>
                </div>

                <div style="display:flex; gap:8px; align-items:center;">
                    <button
                        class="home-button"
                        onclick={set_archive}
                        style={ if is_archive { "" } else { "opacity:0.6; filter:saturate(0.6);" } }
                    >
                        { "Archive" }
                    </button>
                    <button
                        class="home-button"
                        onclick={set_topics}
                        style={ if is_topics { "" } else { "opacity:0.6; filter:saturate(0.6);" } }
                    >
                        { "Topics" }
                    </button>
                </div>
            </header>
        }
    };
    html! {
        <Page {header}>
        {
            if *mode == HomeMode::Archive {
                html! {
                    <ArchiveView
                        toc_items={toc_items.clone()}
                        on_open_post={on_open_post.clone()}
                    />
                }
            } else {
                html! {
                    <>
                        {
                            for topics.iter().map(|(topic, titles)| {
                                let is_open = expanded_topics.contains(topic);
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
                    </>
                }
            }
        }
        </Page>
    }
}
