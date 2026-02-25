use crate::components::card::Card;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AboutMeViewProps {
    pub nickname: String,
    pub avatar_url: String,
    pub bio: String,
    pub github_url: String,
    pub wechat_id: String,
    pub is_open: bool,
    pub on_toggle_panel: Callback<()>,
}

#[function_component(AboutMeView)]
pub fn about_me_view(
    AboutMeViewProps {
        nickname,
        avatar_url,
        bio,
        github_url,
        wechat_id,
        is_open,
        on_toggle_panel,
    }: &AboutMeViewProps,
) -> Html {
    let on_toggle = {
        let on_toggle_panel = on_toggle_panel.clone();
        Callback::from(move |_| on_toggle_panel.emit(()))
    };

    let panel_class = if *is_open {
        classes!("about-panel", "is-open")
    } else {
        classes!("about-panel", "is-collapsed")
    };

    html! {
        <aside class={panel_class}>
            {
                if *is_open {
                    html! {
                        <Card class={classes!("about-card")}>
                            <div class="about-header-row">
                                <h2 class="about-title">{ "About Me" }</h2>
                                <button class="about-toggle" onclick={on_toggle.clone()}>{ "收起" }</button>
                            </div>
                            <div class="about-profile">
                                <img class="about-avatar" src={avatar_url.clone()} alt="avatar" />
                                <div class="about-profile-text">
                                    <div class="about-nickname">{ nickname.clone() }</div>
                                    <p class="about-bio">{ bio.clone() }</p>
                                </div>
                            </div>
                            <ul class="list about-list">
                                <li>
                                    <a class="about-link" href={github_url.clone()} target="_blank" rel="noopener noreferrer">
                                        { "GitHub" }
                                    </a>
                                </li>
                                <li class="about-wechat">
                                    { format!("微信: {}", wechat_id) }
                                </li>
                            </ul>
                        </Card>
                    }
                } else {
                    html! {
                        <Card class={classes!("about-card", "about-card-collapsed")}>
                            <button class="about-toggle about-toggle-collapsed" onclick={on_toggle}>
                                { "关于我" }
                            </button>
                        </Card>
                    }
                }
            }
        </aside>
    }
}
