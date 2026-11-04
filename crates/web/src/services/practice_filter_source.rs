use zipeek_exercises::practice_draft::{DictionaryFilters, HskBand};

pub fn encode(filters: &DictionaryFilters) -> String {
    let hsk = filters.hsk.iter().map(|band| format!("{}.{}", band.system, band.level)).collect::<Vec<_>>().join(",");
    let categories = filters.categories.iter().cloned().collect::<Vec<_>>().join(",");
    let pos = filters.parts_of_speech.iter().cloned().collect::<Vec<_>>().join(",");
    format!("filters:{hsk}|{categories}|{pos}")
}

pub fn decode(source: &str) -> Result<DictionaryFilters, String> {
    if source.len() > 2048 {
        return Err("Practice filters are too long".to_owned());
    }
    let payload = source.strip_prefix("filters:").ok_or("Invalid practice filter source")?;
    let segments = payload.split('|').collect::<Vec<_>>();
    if segments.len() != 3 {
        return Err("Invalid practice filter format".to_owned());
    }
    let mut filters = DictionaryFilters {
        hsk: Default::default(),
        categories: Default::default(),
        parts_of_speech: Default::default(),
    };
    if !segments[0].is_empty() {
        for token in segments[0].split(',') {
            let (system, level) = token.split_once('.').ok_or("Invalid HSK filter")?;
            let band = HskBand::new(
                system.parse().map_err(|_| "Invalid HSK system")?,
                level.parse().map_err(|_| "Invalid HSK level")?,
            ).map_err(|_| "Unsupported HSK filter")?;
            if !filters.hsk.insert(band) { return Err("Duplicate HSK filter".to_owned()); }
        }
    }
    for (segment, output) in [(segments[1], &mut filters.categories), (segments[2], &mut filters.parts_of_speech)] {
        if !segment.is_empty() {
            for token in segment.split(',') {
                if token.is_empty() || token.len() > 64 || !token.bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'_') {
                    return Err("Invalid category or part-of-speech filter".to_owned());
                }
                if !output.insert(token.to_owned()) { return Err("Duplicate vocabulary filter".to_owned()); }
            }
        }
        if output.len() > 64 { return Err("Too many vocabulary filters".to_owned()); }
    }
    if filters.hsk.len() > 16 || (filters.hsk.is_empty() && filters.categories.is_empty() && filters.parts_of_speech.is_empty()) {
        return Err("Choose at least one vocabulary filter".to_owned());
    }
    Ok(filters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_multiple_dimensions() {
        let mut filters = DictionaryFilters::default();
        filters.hsk.insert(HskBand::new(2, 6).unwrap());
        filters.add_category("travel").unwrap();
        filters.parts_of_speech.insert("noun".to_owned());
        assert_eq!(decode(&encode(&filters)), Ok(filters));
    }

    #[test]
    fn invalid_or_unbounded_source_fails_closed() {
        for value in ["filters:", "filters:2.7||", "filters:3.1|travel;DROP|", "filters:3.1,3.1||", "filters:3.1|Travel|", "filters:3.1|travel|noun|extra"] {
            assert!(decode(value).is_err(), "Unexpectedly valid: {value}");
        }
    }
}
