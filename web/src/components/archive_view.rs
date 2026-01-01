use crate::components::{TocItem, card::Card};
use chrono::Datelike;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ArchiveViewProps {
    pub toc_items: Vec<TocItem>,
    pub on_open_post: Callback<String>,
}

fn group_by_year_month(items: &[TocItem]) -> Vec<((i32, u32), Vec<TocItem>)> {
    let mut groups: Vec<((i32, u32), Vec<TocItem>)> = Vec::new();

    for it in items.iter().cloned() {
        let key = (it.date.year(), it.date.month()); // (YYYY, M)
        match groups.last_mut() {
            Some((last_key, bucket)) if *last_key == key => bucket.push(it),
            _ => groups.push((key, vec![it])),
        }
    }

    groups
}

#[function_component(ArchiveView)]
pub fn archive_view(
    ArchiveViewProps {
        toc_items,
        on_open_post,
    }: &ArchiveViewProps,
) -> Html {
    let groups = group_by_year_month(&toc_items);

    let mut last_year: Option<i32> = None;
    html! {
        <>
        {
            for groups.into_iter().map(|((year,month),items)|{
                    let year_header = if last_year != Some(year) {
                        last_year = Some(year);
                        html! { <h2 class="archive-year">{ format!("{year}") }</h2> }
                    } else {
                        html! {}
                    };
                    let month_title = format!("{:02} 月", month);
                    html!{
                        <>
                        {year_header}
                        <Card>
                        <div class="archive-month-row">
                            <div class="archive-month-title">
                                { month_title }
                            </div>
                            <div class="archive-month-count">
                                { format!("{} 篇", items.len()) }
                            </div>
                        </div>
                        <ul class="list">
                            {
                                for items.into_iter().map(|it| {
                                    let path = it.path.clone();
                                    let on_open = {
                                        let cb = on_open_post.clone();
                                        Callback::from(move |_| cb.emit(path.clone()))
                                    };
                                    html!{
                                        <li>
                                            <button onclick={on_open} class="link-button">
                                                <span class="archive-date">
                                                    { it.date.format("%m-%d").to_string() }
                                                </span>
                                                <span>{ it.title }</span>
                                            </button>
                                        </li>
                                    }
                                })
                            }
                        </ul>
                        </Card>
                    </>
                }
            })
        }
        </>
    }
}
