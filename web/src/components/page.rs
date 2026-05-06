use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PageProps {
    pub header: Html,
    #[prop_or_default]
    pub class: Classes,
    pub children: Children,
}

#[function_component(Page)]
pub fn page(
    PageProps {
        header,
        class,
        children,
    }: &PageProps,
) -> Html {
    html! {
        <main class={classes!("page", class.clone())}>
            {header.clone()}
            {for children.iter()}
        </main>
    }
}
