use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CardProps {
    #[prop_or_default]
    pub class: Classes,
    pub children: Children,
}

#[function_component(Card)]
pub fn card(CardProps { class, children }: &CardProps) -> Html {
    html! {
        <section class={classes!("card", class.clone())}>
            { for children.iter() }
        </section>
    }
}
