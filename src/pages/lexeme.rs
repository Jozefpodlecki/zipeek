use log::*;
use yew::prelude::*;
use zipseek_core::{ChineseLexeme, LexicalVariant, OwnedLexemeNeighbors, PartOfSpeech};
use crate::{api::ApiClient, components::{Error, FadeContainer, FadeState, Loaded, Loader}, models::{AppError, Lexeme, LexemeBreakdown}};

#[derive(Clone, Default, Debug)]
pub enum PageState {
    #[default]
    Loading,
    Transitioning(Option<OwnedLexemeNeighbors>),
    Loaded(Option<OwnedLexemeNeighbors>),
    Error(AppError),
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub query: String
}

pub async fn lookup_lexeme(
    lexeme: Box<str>,
    api_client: ApiClient,
    page_state: UseStateSetter<PageState>,
    fade_state: UseStateHandle<FadeState>,
) {
    page_state.set(PageState::Loading);

    match api_client.get_lexeme(&lexeme).await {
        Ok(value) => { PageState::Transitioning(value); },
        Err(err) => { page_state.set(PageState::Error(err)); },
    }

}

#[function_component(LexemePage)]
pub fn lexeme(props: &Props) -> Html {
    let mut api_client: ApiClient = unsafe { use_context::<ApiClient>().unwrap_unchecked() };
    let page_state: UseStateHandle<PageState> = use_state(PageState::default);
    let fade_state: UseStateHandle<FadeState> = use_state(Default::default);

    {
        let api_client = api_client.clone();
        let fade_state = fade_state.clone();
        let page_state: UseStateSetter<PageState> = page_state.setter();
        let query = props.query.clone().into();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(lookup_lexeme(query, api_client, page_state, fade_state));
            || ()
        });
    }

    let on_transition_end = {
        let fade_state = fade_state.clone();
        let page_state = page_state.clone();

        Callback::from(move |_| match *fade_state {
            FadeState::FadingOut => {

                if let PageState::Transitioning(data) = &*page_state {
                    page_state.set(PageState::Loaded(data.clone()));
                }
                
                fade_state.set(FadeState::FadingIn);
            },
            FadeState::FadingIn => fade_state.set(FadeState::Idle),
            _ => {}
        })
    };

    let content = match &*page_state {
        PageState::Loading | PageState::Transitioning(_) => html! { <Loader /> },
        PageState::Error(error) => html!(<Error error={error.clone()}/>),
        PageState::Loaded(value) => html!(<Loaded value={value.clone()}/>) ,
    };

    html! {
        <FadeContainer class="size-full" state={(&*fade_state).clone()} on_transition_end={on_transition_end}>
            {content}
        </FadeContainer>
    }
}
