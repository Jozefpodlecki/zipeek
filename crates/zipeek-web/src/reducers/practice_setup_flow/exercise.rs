pub(super) fn configuration_tab(id: &str) -> &str {
    match id {
        "sentence-translation" => "translation",
        "listening-choice" => "listening",
        "image-vocabulary" => "image",
        "character-hunt" => "hunt",
        "fill-blank" => "fill-blank",
        "audio-dictation" => "audio-dictation",
        "sentence-builder" => "sentence-builder",
        "spot-mistake" => "spot-mistake",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exercise_aliases_use_their_configuration_tabs() {
        assert_eq!(configuration_tab("sentence-translation"), "translation");
        assert_eq!(configuration_tab("listening-choice"), "listening");
        assert_eq!(configuration_tab("image-vocabulary"), "image");
        assert_eq!(configuration_tab("speaking"), "speaking");
        assert_eq!(configuration_tab("matching"), "matching");
    }
}
