use super::{LearningSupport, MapRegion, QuestionKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading<'a> {
    pub title: &'static str,
    pub hanzi: &'a str,
    pub pinyin: &'a str,
    pub english: Option<&'a str>,
    pub show_hanzi: bool,
    pub show_pinyin: bool,
}

pub fn reading<'a>(
    region: &'a MapRegion,
    kind: QuestionKind,
    support: LearningSupport,
    revealed: bool,
    reviewing: bool,
) -> Reading<'a> {
    let title = match kind {
        QuestionKind::FindName => "Find this region",
        QuestionKind::FindCapital => "Find the region with this capital",
        QuestionKind::FindOutline => "Find the region matching this outline",
    };
    let is_outline = kind == QuestionKind::FindOutline;
    Reading {
        title,
        hanzi: if kind == QuestionKind::FindCapital {
            region.capital_hanzi.as_deref().unwrap_or(&region.hanzi)
        } else {
            &region.hanzi
        },
        pinyin: if kind == QuestionKind::FindCapital {
            region.capital_pinyin.as_deref().unwrap_or(&region.pinyin)
        } else {
            &region.pinyin
        },
        english: (reviewing
            || (support == LearningSupport::Guided && kind == QuestionKind::FindName))
            .then_some(region.english.as_str()),
        show_hanzi: !is_outline || reviewing || revealed,
        show_pinyin: reviewing || revealed || (!is_outline && support == LearningSupport::Guided),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::province_quest::RegionKind;

    #[test]
    fn challenge_outline_does_not_expose_region_name() {
        let region = MapRegion {
            id: "53".into(),
            parent_id: None,
            hanzi: "云南".into(),
            pinyin: "Yúnnán".into(),
            english: "Yunnan".into(),
            capital_hanzi: None,
            capital_pinyin: None,
            kind: RegionKind::Province,
            path: "M0,0Z".into(),
            bbox: [0.0, 0.0, 10.0, 10.0],
        };
        let prompt = reading(
            &region,
            QuestionKind::FindOutline,
            LearningSupport::Challenge,
            false,
            false,
        );
        assert!(!prompt.show_hanzi);
        assert!(!prompt.show_pinyin);
        assert!(
            reading(
                &region,
                QuestionKind::FindOutline,
                LearningSupport::Challenge,
                true,
                false
            )
            .show_hanzi
        );
        for support in [
            LearningSupport::Guided,
            LearningSupport::Reading,
            LearningSupport::Challenge,
        ] {
            let hidden = reading(&region, QuestionKind::FindOutline, support, false, false);
            assert!(
                !hidden.show_hanzi,
                "silhouette questions must not disclose the answer in {support:?}"
            );
            assert!(!hidden.show_pinyin);
            assert!(reading(&region, QuestionKind::FindOutline, support, false, true).show_hanzi);
        }
    }
}
