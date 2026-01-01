use crate::components::{TocItem, card::Card};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchViewProps {
    pub toc_items: Vec<TocItem>,
    pub keyword: Option<String>,
    pub on_search: Callback<String>,
    pub on_open_post: Callback<String>,
}

#[function_component(SearchView)]
pub fn search_view(
    SearchViewProps {
        toc_items,
        keyword,
        on_search,
        on_open_post,
    }: &SearchViewProps,
) -> Html {
    let is_open = use_state(|| true);
    let current_keyword = keyword.clone().unwrap_or_default();
    let input_value = use_state(|| current_keyword.clone());

    {
        let input_value = input_value.clone();
        let current_keyword = current_keyword.clone();
        use_effect_with(current_keyword, move |kw| {
            input_value.set(kw.clone());
            || ()
        });
    }

    let on_input = {
        let input_value = input_value.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            input_value.set(value);
        })
    };

    let on_submit = {
        let input_value = input_value.clone();
        let on_search = on_search.clone();
        Callback::from(move |_| {
            on_search.emit((*input_value).clone());
        })
    };
    let on_toggle = {
        let is_open = is_open.clone();
        Callback::from(move |_| is_open.set(!*is_open))
    };

    let normalized = current_keyword.trim().to_lowercase();
    let results: Vec<&TocItem> = if normalized.is_empty() {
        Vec::new()
    } else {
        toc_items
            .iter()
            .filter(|item| item.title.to_lowercase().contains(&normalized))
            .collect()
    };

    let panel_class = if *is_open {
        classes!("search-panel", "is-open")
    } else {
        classes!("search-panel", "is-collapsed")
    };

    html! {
        <aside class={panel_class}>
            {
                if *is_open {
                    html! {
                        <Card class={classes!("search-card")}>
                            <div class="search-header-row">
                                <div class="search-header">
                                    <h2 class="search-title">{ "Search" }</h2>
                                    <p class="search-subtitle">{ "Find posts by title keyword." }</p>
                                </div>
                                <button class="search-toggle" onclick={on_toggle.clone()}>{ "收起" }</button>
                            </div>
                            <div class="search-box">
                                <input
                                    class="search-input"
                                    type="text"
                                    value={(*input_value).clone()}
                                    placeholder="Type a keyword..."
                                    oninput={on_input}
                                />
                                <button class="search-button" onclick={on_submit.clone()}>{ "Go" }</button>
                            </div>
                            <div class="search-results">
                                {
                                    if normalized.is_empty() {
                                        html! { <p class="search-hint">{ "输入关键字后显示结果。" }</p> }
                                    } else if results.is_empty() {
                                        html! { <p class="search-hint">{ "没有匹配的文章。" }</p> }
                                    } else {
                                        html! {
                                            <>
                                                <div class="search-count">
                                                    { format!("{} 篇", results.len()) }
                                                </div>
                                                <ul class="list search-list">
                                                    {
                                                        for results.into_iter().map(|item| {
                                                            let path = item.path.clone();
                                                            let on_open = {
                                                                let cb = on_open_post.clone();
                                                                Callback::from(move |_| cb.emit(path.clone()))
                                                            };
                                                            html! {
                                                                <li>
                                                                    <button onclick={on_open} class="link-button">
                                                                        <span>{ item.title.clone() }</span>
                                                                    </button>
                                                                </li>
                                                            }
                                                        })
                                                    }
                                                </ul>
                                            </>
                                        }
                                    }
                                }
                            </div>
                        </Card>
                    }
                } else {
                    html! {
                        <Card class={classes!("search-card", "search-card-collapsed")}>
                            <button class="search-toggle search-toggle-collapsed" onclick={on_toggle}>
                                { "展开" }
                            </button>
                        </Card>
                    }
                }
            }
        </aside>
    }
}
