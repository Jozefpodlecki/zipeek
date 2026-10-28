use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RoutePlaceholderProps {
    pub title: AttrValue,
    pub message: AttrValue,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(RoutePlaceholder)]
pub fn route_placeholder(props: &RoutePlaceholderProps) -> Html {
    html! {
        <section data-route-placeholder="" class="flex min-h-72 flex-col items-start justify-center md:min-h-96">
            <h1 class="text-[clamp(2.5rem,5vw,3.5rem)] leading-[1.05] font-bold tracking-[-.055em]">{props.title.clone()}</h1>
            <p class="mt-6 text-muted">{props.message.clone()}</p>
            {for props.children.iter()}
        </section>
    }
}
