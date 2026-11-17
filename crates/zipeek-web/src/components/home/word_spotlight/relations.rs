pub(super) fn relation_label(relation_type: &str) -> &str {
    match relation_type {
        "abbreviation_of" => "Abbreviation of",
        "short_for" => "Short for",
        "variant_of" => "Variant of",
        "also_written_as" => "Also written as",
        "see_also" => "See also",
        _ => "Reference",
    }
}
