use yew::*;

#[derive(PartialEq, Default, Clone, Copy)]
pub enum FadeState {
    #[default]
    Idle,
    FadingOut,
    FadingIn,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
    pub children: Children,
    pub state: FadeState,
    #[prop_or(200)]
    pub duration: u32,
    pub on_transition_end: Callback<TransitionEvent>
}

#[function_component(FadeContainer)]
pub fn fade_container(props: &Props) -> Html {

    let opacity_class = match props.state {
        FadeState::FadingOut => format!("{} opacity-0 transition-opacity", props.class),
        FadeState::FadingIn | FadeState::Idle => format!("{} opacity-100 transition-opacity", props.class),
    };

    html! {
        <div
            data-fade-container=""
            class={opacity_class}
            style={format!("transition-duration: {}ms;", props.duration)}
            ontransitionend={&props.on_transition_end}>
            { for props.children.iter() }
        </div>
    }
}