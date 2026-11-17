use crate::preferences::PreferencesStore;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HanziFont {
    pub key: &'static str,
    pub name: &'static str,
}

pub(crate) const HANZI_FONTS: [HanziFont; 10] = [
    HanziFont {
        key: "system",
        name: "System",
    },
    HanziFont {
        key: "sans",
        name: "Sans-serif",
    },
    HanziFont {
        key: "serif",
        name: "Serif",
    },
    HanziFont {
        key: "calligraphy",
        name: "Calligraphy",
    },
    HanziFont {
        key: "lxgw-wenkai",
        name: "LXGW WenKai",
    },
    HanziFont {
        key: "ma-shan-zheng",
        name: "Ma Shan Zheng",
    },
    HanziFont {
        key: "zcool-xiaowei",
        name: "ZCOOL XiaoWei",
    },
    HanziFont {
        key: "long-cang",
        name: "Long Cang",
    },
    HanziFont {
        key: "noto-serif-sc",
        name: "Noto Serif SC",
    },
    HanziFont {
        key: "zcool-kuaile",
        name: "ZCOOL KuaiLe",
    },
];

pub(crate) fn font_name(key: &str) -> &'static str {
    HANZI_FONTS
        .iter()
        .find(|font| font.key == key)
        .map(|font| font.name)
        .unwrap_or("System")
}

pub(crate) fn stored_font(storage: &web_sys::Storage) -> String {
    let value = PreferencesStore::new(storage).read().font;
    if HANZI_FONTS.iter().any(|font| font.key == value) {
        value
    } else {
        "system".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_font_has_distinct_persisted_key() {
        let mut keys = std::collections::BTreeSet::new();
        for font in HANZI_FONTS {
            assert!(keys.insert(font.key));
            assert!(!font.name.is_empty());
        }
        assert_eq!(keys.len(), 10);
    }

    #[test]
    fn unknown_font_is_not_misrepresented() {
        assert_eq!(font_name("invalid"), "System");
        assert_eq!(font_name("lxgw-wenkai"), "LXGW WenKai");
    }
}

pub(crate) const CHARACTER_SIZES: [(&str, &str); 3] = [
    ("normal", "Standard"),
    ("large", "Large"),
    ("extra-large", "Extra large"),
];

pub(crate) fn character_size(value: &str) -> &'static str {
    CHARACTER_SIZES
        .iter()
        .find(|(key, _)| *key == value)
        .map(|(key, _)| *key)
        .unwrap_or("normal")
}

#[cfg(test)]
mod size_tests {
    use super::*;

    #[test]
    fn unsupported_sizes_return_safe_default() {
        assert_eq!(character_size("huge"), "normal");
        assert_eq!(character_size("large"), "large");
    }
}
