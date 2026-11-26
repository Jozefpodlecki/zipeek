#[derive(Clone, Copy)]
pub struct SoundGroup {
    pub label: &'static str,
    pub symbols: &'static str,
    pub note: &'static str,
}

const INITIALS: &[SoundGroup] = &[
    SoundGroup {
        label: "Stops and affricates",
        symbols: "b p · d t · g k · j q · zh ch · z c",
        note: "The pairs contrast unaspirated and aspirated releases.",
    },
    SoundGroup {
        label: "Nasals and liquids",
        symbols: "m n l r",
        note: "Pay attention to the tongue position of n, l and r.",
    },
    SoundGroup {
        label: "Fricatives",
        symbols: "f h x sh s",
        note: "Contrast alveolar, retroflex and palatal articulation.",
    },
    SoundGroup {
        label: "Syllable spelling",
        symbols: "y w",
        note: "y and w are spelling conventions used with certain zero-initial syllables, not additional consonant initials in the traditional 21-initial inventory.",
    },
];

const VOWELS: &[SoundGroup] = &[
    SoundGroup {
        label: "Core written vowels",
        symbols: "a · o · e · i · u · ü",
        note: "The actual sound of a vowel depends on the final and surrounding sounds.",
    },
    SoundGroup {
        label: "Front rounded vowel",
        symbols: "ü · lǜ · nǚ",
        note: "Round your lips while keeping the tongue toward the front of the mouth.",
    },
];

const COMPOUNDS: &[SoundGroup] = &[
    SoundGroup {
        label: "Common compound finals",
        symbols: "ai · ei · ao · ou · ia · ie · ua · uo · üe · er",
        note: "Pronounce each as a syllabic unit rather than spelling out letters.",
    },
    SoundGroup {
        label: "Additional combinations",
        symbols: "iao · iou (iu) · uai · uei (ui)",
        note: "Some full finals have shortened written forms after an initial.",
    },
];

const NASALS: &[SoundGroup] = &[
    SoundGroup {
        label: "Front nasal finals",
        symbols: "an · en · ian · in · uan · uen (un) · üan · ün",
        note: "Front-nasal finals end in -n.",
    },
    SoundGroup {
        label: "Back nasal finals",
        symbols: "ang · eng · ong · iang · ing · iong · uang · ueng",
        note: "Back-nasal finals end in -ng, with vowel-quality changes in context.",
    },
];

const SPELLING: &[SoundGroup] = &[
    SoundGroup {
        label: "The ü spelling rule",
        symbols: "ju · qu · xu · yu · nü · lü",
        note: "After j, q, x and in yu, the two dots are omitted; after n and l they remain.",
    },
    SoundGroup {
        label: "Tone marks",
        symbols: "ā á ǎ à · ē é ě è · ī í ǐ ì",
        note: "Tone marks normally go on a, then o/e, otherwise the last vowel in iu and ui.",
    },
    SoundGroup {
        label: "Syllable boundaries",
        symbols: "Xī'ān · pí'ǎo",
        note: "Use an apostrophe before a vowel-initial syllable when boundaries could be ambiguous.",
    },
];

pub fn groups_for(module: usize) -> &'static [SoundGroup] {
    match module {
        2 | 3 => INITIALS,
        4 => VOWELS,
        5 => COMPOUNDS,
        6 => NASALS,
        7 | 8 => SPELLING,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provides_initial_and_final_inventory() {
        assert!(
            groups_for(2)
                .iter()
                .any(|item| item.symbols.contains("zh ch"))
        );
        assert!(groups_for(4).iter().any(|item| item.symbols.contains('ü')));
        assert!(
            groups_for(6)
                .iter()
                .any(|item| item.symbols.contains("uang"))
        );
        assert!(groups_for(11).is_empty());
    }
}
