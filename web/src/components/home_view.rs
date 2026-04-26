use crate::components::webgl_ring::WebglRing;
use crate::components::{TocItem, archive_view::ArchiveView, page::Page, topic_card::TopicCard};
use std::collections::{HashMap, HashSet};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HomeViewProps {
    pub toc_items: Vec<TocItem>,
    pub topics: Vec<(String, Vec<String>)>,
    pub series: Vec<(String, Vec<String>)>,
    pub title_to_path: HashMap<String, String>,
    pub expanded_topics: HashSet<String>,
    pub expanded_series: HashSet<String>,

    pub on_toggle_topic: Callback<String>,
    pub on_toggle_series: Callback<String>,
    pub on_open_post: Callback<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HomeMode {
    Archive,
    Topics,
    Series,
    Ring,
}
#[function_component(HomeView)]
pub fn home_view(
    HomeViewProps {
        toc_items,
        topics,
        series,
        title_to_path,
        expanded_topics,
        expanded_series,
        on_toggle_topic,
        on_toggle_series,
        on_open_post,
    }: &HomeViewProps,
) -> Html {
    let mode = use_state(|| HomeMode::Ring);
    let set_archive = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(HomeMode::Archive))
    };
    let set_topics = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(HomeMode::Topics))
    };
    let set_ring = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(HomeMode::Ring))
    };
    let set_series = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(HomeMode::Series))
    };
    let header = {
        let is_archive = *mode == HomeMode::Archive;
        let is_topics = *mode == HomeMode::Topics;
        let is_series = *mode == HomeMode::Series;
        let is_ring = *mode == HomeMode::Ring;

        html! {
            <header class="header">
                <div>
                    <h1 class="title">{ "Home" }</h1>
                    <p class="subtitle">
                        {
                            if is_archive { "Archive (by date)" }
                            else if is_topics { "Browse by topic" }
                            else if is_series { "Browse by series" }
                            else { "Planet Ring" }
                        }
                    </p>
                </div>

                <div class="home-mode-toggle">
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
                    <button
                        class="home-button"
                        onclick={set_series}
                        style={ if is_series { "" } else { "opacity:0.6; filter:saturate(0.6);" } }
                    >
                        { "Series" }
                    </button>
                    <button
                        class="home-button"
                        onclick={set_ring}
                        style={ if is_ring { "" } else { "opacity:0.6; filter:saturate(0.6);" } }
                    >
                        { "Ring" }
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
            } else if *mode == HomeMode::Ring {
                html! {
                    <WebglRing
                        toc_items={toc_items.clone()}
                        on_open_post={on_open_post.clone()}
                    />
                }
            } else if *mode == HomeMode::Series {
                html! {
                    <>
                        {
                            for series.iter().map(|(series_name, titles)| {
                                let is_open = expanded_series.contains(series_name);
                                html! {
                                    <TopicCard
                                        label={series_name.clone()}
                                        titles={titles.clone()}
                                        title_to_path={title_to_path.clone()}
                                        is_open={is_open}
                                        on_toggle={on_toggle_series.clone()}
                                        on_open_post={on_open_post.clone()}
                                    />
                                }
                            })
                        }
                    </>
                }
            } else {
                html! {
                    <>
                        {
                            for topics.iter().map(|(topic, titles)| {
                                let is_open = expanded_topics.contains(topic);
                                html! {
                                    <TopicCard
                                        label={topic.clone()}
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
