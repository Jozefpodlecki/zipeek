use gloo::timers::callback::Timeout;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_icons::{Icon, IconData};
use log::*;
use crate::{components::{Background, Footer}, contexts::SearchContext, models::Social, services::ApiClient};


#[derive(Properties, PartialEq)]
pub struct SearchProps {
    pub on_change: Callback<String>,
    pub class: String,
}

#[function_component(SearchBar)]
pub fn searchbar(props: &SearchProps) -> Html {
    let value: UseStateHandle<String> = use_state(Default::default);
    let timeout = use_mut_ref(|| Option::<Timeout>::None);

    let on_input: Callback<InputEvent> = {
        let on_change = props.on_change.clone();
        let value_state = value.clone();
        let timeout = timeout.clone();

        Callback::from(move |event: InputEvent| {
            let on_change = on_change.clone();
            let element: HtmlInputElement = event.target().unwrap().unchecked_into();
            let new_value = element.value();
            value_state.set(new_value.clone());

            if let Some(timeout) = timeout.borrow_mut().take() {
                timeout.cancel();
            }

            let handle = Timeout::new(500, move || {
                on_change.emit(new_value);
            });

            *timeout.borrow_mut() = Some(handle);
        })
    };

    let on_clear: Callback<MouseEvent> = {
        let on_change = props.on_change.clone();
        let value_state = value.clone();
        let timeout = timeout.clone();

        Callback::from(move |event: MouseEvent| {
            let on_change = on_change.clone();

            if let Some(timeout) = timeout.borrow_mut().take() {
                timeout.cancel();
            }

            value_state.set(Default::default());
            on_change.emit(Default::default());
        })
    };

    html! {
        <div class={format!("{} flex justify-center", props.class)}>
            <input
                type="text"
                value={(&*value).clone()}
                oninput={on_input}
                placeholder="Search for lexeme/phrase..."
                class="w-full text-3xl text-white border-transparent focus:border-transparent focus:ring-0 outline-none p-2" />
                <button
                    disabled={(*value).is_empty()}
                    type="button"
                    class="
                        transition-all
                        text-white
                        hover:opacity-50

                        disabled:opacity-30
                        disabled:hover:opacity-30
                    "
                    onclick={on_clear}
                >
                    <Icon data={IconData::LUCIDE_X} width={"20px"} />
                </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Layout)]
pub fn layout(props: &Props) -> Html {
    let social = use_context::<Social>();
    let search_context = use_context::<SearchContext>();

    let on_search: Callback<String> = {
        let search_context = search_context.clone();

        Callback::from(move |value: String| {
            let search_context = search_context.clone();
            search_context.unwrap().set(value);
        })
    };

    html! {
        <article data-layout="" class="size-full flex flex-col min-h-screen">
            <header class="flex gap-4 justify-center items-center text-white p-2">
                <img src="public/favicon-32x32.png" class="" alt="logo"/>
                <span class="font-[roboto] text-4xl">{"Zipeek"}</span>
                
            </header>
            <div class="flex justify-center">
                <SearchBar class="w-100 mt-10" on_change={on_search} />
            </div>
            <main data-content="" class="flex-1">
                { for props.children.iter() }
            </main>
            { if let Some(social) = social {  html! { <Footer social={social}/> } } else { html!{} } }
        </article>
    }
}