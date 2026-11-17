pub struct CourseInfo {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub icon: &'static str,
    pub accent: &'static str,
}

pub const COURSES: [CourseInfo; 6] = [
    CourseInfo {
        id: "pinyin",
        title: "Pinyin & Pronunciation",
        subtitle: "Learn initials, finals and Mandarin tones",
        icon: "ā á ǎ à",
        accent: "#dbeafe",
    },
    CourseInfo {
        id: "everyday",
        title: "Everyday Chinese",
        subtitle: "Useful phrases for familiar situations",
        icon: "你好",
        accent: "#ffe6d5",
    },
    CourseInfo {
        id: "topics",
        title: "Topics & Vocabulary",
        subtitle: "Vocabulary for food, travel and hobbies",
        icon: "词",
        accent: "#d9f5e8",
    },
    CourseInfo {
        id: "dialogues",
        title: "Dialogues & Scenarios",
        subtitle: "Real-life conversational exchanges",
        icon: "对话",
        accent: "#ebe5ff",
    },
    CourseInfo {
        id: "grammar",
        title: "Grammar Essentials",
        subtitle: "Sentence patterns and their use",
        icon: "的",
        accent: "#fff0c9",
    },
    CourseInfo {
        id: "hsk",
        title: "HSK Preparation",
        subtitle: "Structured practice by proficiency level",
        icon: "HSK",
        accent: "#d9e9ff",
    },
];

pub struct Lesson {
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub explanation: &'static str,
    pub examples: &'static [(&'static str, &'static str)],
}

pub const PINYIN_LESSONS: [Lesson; 12] = [
    Lesson {
        slug: "tones",
        title: "The four tones",
        summary: "Hear pitch as part of the syllable",
        explanation: "Mandarin uses four primary tones and a neutral tone. A tone changes the syllable's meaning, so it is part of pronunciation rather than decoration. The first stays high, the second rises, the third dips then rises, and the fourth falls sharply.",
        examples: &[
            ("mā 妈 — mother", "First tone: high and level"),
            ("má 麻 — hemp", "Second tone: rising"),
            ("mǎ 马 — horse", "Third tone: low, dipping"),
            ("mà 骂 — scold", "Fourth tone: falling"),
        ],
    },
    Lesson {
        slug: "tone-patterns",
        title: "Tone pairs & neutral tone",
        summary: "Notice tone changes in connected speech",
        explanation: "When syllables combine, tones influence each other. Third-tone pairs often change in connected speech, and the neutral tone becomes shorter and lighter than the main stressed syllable. This lesson trains your ear to hear those patterns rather than reading every tone mark in isolation.",
        examples: &[
            ("nǐ hǎo 你好", "Third tone before third tone becomes rising"),
            ("māma 妈妈", "Second syllable is usually neutral"),
            ("bú shì 不是", "不 changes before a fourth tone"),
            ("yí ge 一个", "一 changes before other tones"),
        ],
    },
    Lesson {
        slug: "initials",
        title: "Initial sounds",
        summary: "Understand the start of a syllable",
        explanation: "An initial is the consonant at the beginning of a Mandarin syllable. Some syllables have no initial. Pinyin letters do not always sound like their English equivalents, so practise them as sound categories rather than letter names.",
        examples: &[
            ("bā 八 — eight", "Initial b"),
            ("mā 妈 — mother", "Initial m"),
            ("dōng 东 — east", "Initial d"),
            ("fàn 饭 — rice, meal", "Initial f"),
        ],
    },
    Lesson {
        slug: "initial-contrasts",
        title: "Tricky initials",
        summary: "Distinguish the look-alike consonant groups",
        explanation: "Several Mandarin initials are easiest to learn in families: j, q, x; zh, ch, sh; and the sound r. These contrasts matter more than English spelling intuition. Focus on tongue position and aspiration rather than trying to map them directly to English letters.",
        examples: &[
            ("qī 七 — seven", "Aspirated palatal q"),
            ("xī 西 — west", "Soft palatal fricative x"),
            ("shū 书 — book", "Retroflex sh"),
            ("rì 日 — sun, day", "Mandarin r"),
        ],
    },
    Lesson {
        slug: "simple-finals",
        title: "Simple finals and vowel positions",
        summary: "Recognize the six core vowel sounds",
        explanation: "Mandarin vowels a, o, e, i, u and ü are useful starting points, but their exact sound changes with context. The rounded front vowel ü is distinct from u. Hear each in a full syllable and pay attention to lip rounding.",
        examples: &[
            ("ā 啊 — ah", "Open vowel a"),
            ("yī 一 — one", "High front vowel i"),
            ("wǔ 五 — five", "Rounded back vowel u"),
            ("lǜ 绿 — green", "Rounded front vowel ü"),
        ],
    },
    Lesson {
        slug: "finals",
        title: "Finals and syllables",
        summary: "Combine initials with finals",
        explanation: "A final is the part of a Mandarin syllable after its initial. Finals can contain a vowel, a vowel combination, or a nasal ending. The tone belongs to the syllable as a whole. This is where pinyin starts to feel like full syllables instead of isolated sounds.",
        examples: &[
            ("mā = m + ā", "Initial plus simple final"),
            ("hǎo = h + ǎo", "Initial plus compound final"),
            ("zhōng = zh + ōng", "Initial plus nasal final"),
            ("ài = ài", "No initial"),
        ],
    },
    Lesson {
        slug: "nasal-finals",
        title: "Nasal finals",
        summary: "Hear the difference between -n and -ng",
        explanation: "Mandarin nasal finals distinguish a front nasal ending -n and a back nasal ending -ng. This contrast occurs in many vowel combinations. Listen to the whole syllable and attend to where the tongue closes.",
        examples: &[
            ("ān 安 — peaceful", "Nasal final an"),
            ("āng 肮 — dirty (as in 肮脏)", "Nasal final ang"),
            ("lín 林 — forest", "Ends in -n"),
            ("líng 零 — zero", "Ends in -ng"),
        ],
    },
    Lesson {
        slug: "spelling-review",
        title: "Spelling rules & review",
        summary: "Read full pinyin syllables with confidence",
        explanation: "This review lesson ties together tones, initials and finals. It also introduces common spelling shortcuts in pinyin, such as the way umlaut vowels are written after j, q and x, or how apostrophes help separate syllables. The goal is to read and write syllables cleanly.",
        examples: &[
            ("lǜ 绿 — green", "ü after l keeps the umlaut"),
            ("qù 去 — go", "ü after q is written as u"),
            ("Xi'an 西安", "Apostrophe separates syllables"),
            ("yuè 月 — moon", "Complete syllable reading"),
        ],
    },
    Lesson {
        slug: "syllable-building",
        title: "Build complete syllables",
        summary: "Combine initials, finals and tone marks",
        explanation: "Most Mandarin syllables combine an optional initial, a final and one tone. Some begin with a vowel, while others use y and w spelling conventions. Distinguish the spoken syllable from its written Pinyin form.",
        examples: &[
            ("Zhōngguó 中国 — China", "Two syllables with different tones"),
            ("ér 儿 — child", "Vowel-initial syllable"),
            ("yǔ 雨 — rain", "Initial-free syllable with y spelling"),
            ("huā 花 — flower", "Initial h with the final ua"),
        ],
    },
    Lesson {
        slug: "connected-speech",
        title: "Connected speech and rhythm",
        summary: "Listen for neutral tone and natural phrasing",
        explanation: "Connected Mandarin often changes pitch and syllable prominence. Neutral-tone syllables are typically shorter and less prominent. Third-tone sandhi changes pitch before another third tone; 一 and 不 also change their tone in common environments.",
        examples: &[
            ("nǐ hǎo 你好 — hello", "Third-tone sandhi"),
            ("māma 妈妈 — mother", "Second syllable is neutral"),
            ("bú shì 不是 — is not", "不 changes before fourth tone"),
            ("yí ge 一个 — one item", "一 changes with context"),
        ],
    },
    Lesson {
        slug: "sound-contrasts",
        title: "Sound contrast lab",
        summary: "Distinguish easily confused syllables",
        explanation: "Listen for aspiration, tongue position, vowel rounding and nasal endings. Mandarin b/p, d/t, g/k and z/c are primarily aspiration contrasts. Compare front and back nasal finals and ü with u.",
        examples: &[
            ("bā 八 — eight", "Unaspirated b"),
            ("pā 趴 — lie face down", "Aspirated p"),
            ("lín 林 — forest", "Final -n"),
            ("líng 零 — zero", "Final -ng"),
        ],
    },
    Lesson {
        slug: "comprehensive-review",
        title: "Pinyin review and self-check",
        summary: "Apply tones, initials, finals and spelling",
        explanation: "Review the core Mandarin sound and Pinyin spelling patterns together. Use audio playback to hear examples, then complete the independent written recognition checks. Browser speech is supplementary and does not measure your spoken pronunciation.",
        examples: &[
            ("mǎ 马 — horse", "Third tone"),
            ("qù 去 — go", "ü represented by u after q"),
            ("Xī'ān 西安 — Xi'an", "Apostrophe marks syllable boundary"),
            ("zhōng 中 — middle", "Retroflex initial and nasal final"),
        ],
    },
];
