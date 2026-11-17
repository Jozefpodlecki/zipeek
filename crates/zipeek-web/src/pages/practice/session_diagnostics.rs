use super::*;

pub(super) fn visible_input(status: &PracticeStatus) -> String {
    match status {
        PracticeStatus::Ready(prepared) => match &prepared.round.content {
            RoundContent::Matching { left, right } => format!(
                "Matching: {} / {}",
                left.iter()
                    .map(|card| card.label.as_str())
                    .collect::<Vec<_>>()
                    .join(" · "),
                right
                    .iter()
                    .map(|card| card.label.as_str())
                    .collect::<Vec<_>>()
                    .join(" · "),
            ),
            RoundContent::Mcq {
                prompt, options, ..
            } => format!(
                "MCQ: {} / {}",
                prompt.label,
                options
                    .iter()
                    .map(|card| card.label.as_str())
                    .collect::<Vec<_>>()
                    .join(" · "),
            ),
        },
        _ => String::new(),
    }
}

pub(super) fn debug_round(status: &PracticeStatus) -> Option<String> {
    match status {
        PracticeStatus::Ready(prepared) if cfg!(debug_assertions) => {
            let cards = match &prepared.round.content {
                RoundContent::Matching { left, right } => {
                    left.iter().chain(right.iter()).collect::<Vec<_>>()
                }
                RoundContent::Mcq {
                    prompt, options, ..
                } => std::iter::once(prompt).chain(options.iter()).collect(),
            };
            serde_json::to_string(
                &cards
                    .iter()
                    .map(|card| (card.word_id.as_i32(), card.label.as_str()))
                    .collect::<Vec<_>>(),
            )
            .map_err(|error| log::error!("Could not encode round diagnostics: {error}"))
            .ok()
        }
        _ => None,
    }
}
