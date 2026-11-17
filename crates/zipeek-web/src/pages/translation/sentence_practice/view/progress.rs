use super::super::state::TranslationController;
use yew::prelude::*;

pub(super) fn render(state: &TranslationController) -> Html {
    let round_limit_status = {
        if let Some(limit) = state.round_limit {
            format!(
                "Translation exercises: {} / {limit}",
                state.session.completed
            )
        } else {
            format!(
                "Translation exercises: {} completed",
                state.session.completed
            )
        }
    };
    html! { <>
        <div
            data-component="translation-session-stats"
            class="flex flex-wrap gap-4 rounded-lg border border-line bg-surface p-3 text-sm"
            role="status"
        >
            <span>
                {format!("Completed: {}", state.quiz_session.session().entries.len())}
            </span>
            <span>
                {format!("Correct: {}", state.quiz_session.session().totals().earned)}
            </span>
            <span>
                {format!("Mistakes: {}", state.quiz_session.session().totals().mistakes)}
            </span>
        </div>
        <p
            class="text-sm text-muted"
        >
            {"Sentence generation uses words from the selected source. HSK and saved sets are read from the installed local dictionary."}
        </p>
        <p
            data-translation-progress=""
            class="text-sm font-semibold"
        >
            {round_limit_status}
        </p>
    </> }
}
