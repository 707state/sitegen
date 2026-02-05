use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PageProps {
    pub header: Html,
    pub children: Children,
}

#[function_component(Page)]
pub fn page(PageProps { header, children }: &PageProps) -> Html {
    html! {
        <main class="page">
            {header.clone()}
            {for children.iter()}
        </main>
    }
}
