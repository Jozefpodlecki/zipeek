mod exercise;
mod frame;
mod generation;
mod history;
mod page;
mod persistence;
mod progress;
mod registry;
mod restore;
mod state;
mod summary;
#[cfg(test)]
mod tests;
mod transition;

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LocalPracticeSessionProps {
    pub id: String,
}

#[function_component(LocalPracticeSession)]
pub fn local_practice_session(props: &LocalPracticeSessionProps) -> Html {
    html! { <page::LocalPracticeSessionPage key={props.id.clone()} id={props.id.clone()} /> }
}
