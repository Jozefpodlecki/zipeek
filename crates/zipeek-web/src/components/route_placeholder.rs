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
        <section
            data-component="route-placeholder"
            data-route-placeholder=""
            class="flex min-h-72 items-center justify-center px-4 py-10 md:min-h-96"
        >
            <div
                class="w-full max-w-2xl rounded-3xl border border-line bg-surface/90 px-6 py-10 text-center shadow-xl sm:px-10 sm:py-12"
            >
                <h1
                    class="text-[clamp(2rem,4vw,3rem)] leading-[1.1] font-bold tracking-tight text-ink"
                >
                    {props.title.clone()}
                </h1>
                <p
                    class="mx-auto mt-4 max-w-lg text-base leading-relaxed text-muted"
                >
                    {props.message.clone()}
                </p>
                <div
                    class="mt-7 flex flex-wrap items-center justify-center gap-3"
                >
                    {for props.children.iter()}
                </div>
            </div>
        </section>
    }
}
