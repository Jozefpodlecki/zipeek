use yew::prelude::*;
use yew_router::prelude::use_location;
use zipeek_core::practice::PracticeMode;

use crate::components::Loader;

pub use zipeek_core::ExerciseVariant as Variant;

#[derive(Properties, PartialEq)]
pub struct PracticeProps {
    pub variant: Variant,
}

#[function_component(Practice)]
pub fn practice(props: &PracticeProps) -> Html {
    let location = use_location();
    let query = location
        .as_ref()
        .map(crate::services::practice_url::read_query)
        .unwrap_or_default();
    use_effect_with(query.warnings.clone(), |warnings| {
        if cfg!(debug_assertions) {
            for warning in warnings {
                log::warn!("Invalid practice {}: {}", warning.field, warning.value);
            }
        }
        || ()
    });
    let mode = match query.mode {
        PracticeMode::Endless => "endless",
        PracticeMode::Fixed(_) => "fixed",
    };
    html! {
        <div data-practice-variant={props.variant.seed_token()}
            data-practice-mode={mode}
            data-practice-level={query.filter.level.as_i32().to_string()}
            data-practice-offset={query.offset.to_string()}
            data-practice-pos={query.filter.pos_tokens()}>
            <Loader />
        </div>
    }
}
