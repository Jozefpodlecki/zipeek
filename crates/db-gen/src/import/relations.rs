#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalReference {
    pub definition: String,
    pub relation_type: &'static str,
    pub target_simplified: String,
    pub target_traditional: Option<String>,
    pub target_pinyin: Option<String>,
    pub source_annotation: String,
}

pub fn lexical_reference(gloss: &str) -> Option<LexicalReference> {
    const MARKERS: [(&str, &str); 5] = [
        ("abbr. for ", "abbreviation_of"),
        ("short for ", "short_for"),
        ("variant of ", "variant_of"),
        ("also written ", "also_written_as"),
        ("see also ", "see_also"),
    ];
    for (marker, relation_type) in MARKERS {
        let (definition, annotation) = if let Some(prefix) = gloss.strip_prefix(marker) {
            ("", prefix)
        } else if let Some((definition, suffix)) = gloss.rsplit_once(&format!(" ({marker}")) {
            let Some(suffix) = suffix.strip_suffix(')') else { continue };
            (definition.trim(), suffix)
        } else {
            continue;
        };
        let annotation = annotation.trim();
        if annotation.is_empty() || annotation.contains('(') || annotation.contains(')')
            || annotation.contains(" + ") || annotation.contains('/') || annotation.contains(';')
            || annotation.contains(',') || annotation.contains(" or ")
        {
            return None;
        }
        let (characters, pinyin) = match annotation.split_once('[') {
            Some((characters, reading)) => {
                let reading = reading.strip_suffix(']')?.trim();
                if reading.is_empty() || reading.contains('[') { return None; }
                (characters.trim(), Some(reading.to_owned()))
            }
            None => (annotation, None),
        };
        let (traditional, simplified) = match characters.split_once('|') {
            Some((traditional, simplified)) => (Some(traditional.trim()), simplified.trim()),
            None => (None, characters.trim()),
        };
        if simplified.is_empty() || !is_chinese_reference(simplified)
            || traditional.is_some_and(|value| value.is_empty() || !is_chinese_reference(value))
        {
            return None;
        }
        let source_annotation = if definition.is_empty() {
            gloss.to_owned()
        } else {
            format!("({marker}{annotation})")
        };
        return Some(LexicalReference {
            definition: definition.to_owned(),
            relation_type,
            target_simplified: simplified.to_owned(),
            target_traditional: traditional.map(str::to_owned),
            target_pinyin: pinyin,
            source_annotation,
        });
    }
    None
}

fn is_chinese_reference(text: &str) -> bool {
    text.chars().any(is_han_character) && text.chars().all(|ch| is_han_character(ch) || ch.is_ascii_alphanumeric())
}

fn is_han_character(ch: char) -> bool {
    matches!(ch as u32, 0x3400..=0x9fff | 0xf900..=0xfaff | 0x20000..=0x2fa1f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_explicit_abbreviations_and_preserves_clean_definition() {
        let parsed = lexical_reference("supermarket (abbr. for 超级市场)").unwrap();
        assert_eq!(parsed.definition, "supermarket");
        assert_eq!(parsed.relation_type, "abbreviation_of");
        assert_eq!(parsed.target_simplified, "超级市场");
        assert_eq!(parsed.target_traditional, None);
        assert_eq!(parsed.source_annotation, "(abbr. for 超级市场)");
    }

    #[test]
    fn extracts_reading_and_traditional_form() {
        let with_latin = lexical_reference("B-mode ultrasound (abbr. for B型超聲|B型超声[B xing2 chao1 sheng1])").unwrap();
        assert_eq!(with_latin.target_simplified, "B型超声");
        let parsed = lexical_reference("verse (abbr. for 七言律詩|七言律诗[qi1 yan2 lv4 shi1])").unwrap();
        assert_eq!(parsed.target_traditional.as_deref(), Some("七言律詩"));
        assert_eq!(parsed.target_pinyin.as_deref(), Some("qi1 yan2 lv4 shi1"));
    }

    #[test]
    fn supports_explicit_variants_short_forms_and_cross_references() {
        for (gloss, relation_type, expected) in [
            ("alternate spelling (variant of 語言|语言[yu3 yan2])", "variant_of", "语言"),
            ("short for 中华人民共和国", "short_for", "中华人民共和国"),
            ("also written 臺灣|台湾", "also_written_as", "台湾"),
            ("dictionary entry (see also 學習|学习)", "see_also", "学习"),
        ] {
            let reference = lexical_reference(gloss).unwrap();
            assert_eq!(reference.relation_type, relation_type);
            assert_eq!(reference.target_simplified, expected);
        }
    }

    #[test]
    fn does_not_infer_ambiguous_or_unrelated_annotations() {
        for gloss in ["bank (of a river)", "Gen Z (abbr. for 95后 + 00后)",
            "value (abbr. for value)", "term (abbr. for )", "noun (abbr. for",
            "word (see also 你好/再见)", "word (variant of 你好 or 再见)",
            "word (also written 你好, 再见)", "word (short for 甲;乙)",
        ] {
            assert_eq!(lexical_reference(gloss), None, "{gloss}");
        }
    }
}
