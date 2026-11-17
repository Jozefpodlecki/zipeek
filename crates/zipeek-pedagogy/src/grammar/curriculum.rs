pub const AUTHORED_GRAMMAR_SLUGS: [&str; 5] = [
    "intro", "time-place", "possession-quantity", "descriptions", "wants-abilities",
];

pub fn authored_grammar_slug(region: usize) -> Option<&'static str> {
    region.checked_sub(1).and_then(|index| AUTHORED_GRAMMAR_SLUGS.get(index).copied())
}

pub fn authored_grammar_region_from_slug(slug: &str) -> Option<usize> {
    AUTHORED_GRAMMAR_SLUGS.iter().position(|candidate| *candidate == slug).map(|index| index + 1)
}

pub fn authored_grammar_title_for_slug(slug: &str) -> Option<&'static str> {
    match authored_grammar_region_from_slug(slug)? {
        1 => Some(INTRO.title),
        2 => Some(TIME_PLACE.title),
        3 => Some(POSSESSION_QUANTITY.title),
        4 => Some(DESCRIPTIONS.title),
        5 => Some(WANTS_ABILITIES.title),
        _ => None,
    }
}

#[cfg(test)]
mod route_slug_tests {
    use super::*;

    #[test]
    fn published_grammar_slugs_are_unique_and_reversible() {
        for region in 1..=AUTHORED_GRAMMAR_SLUGS.len() {
            let slug = authored_grammar_slug(region).unwrap();
            assert_eq!(authored_grammar_region_from_slug(slug), Some(region));
            assert!(authored_grammar_title_for_slug(slug).is_some());
        }
        assert_eq!(authored_grammar_slug(0), None);
        assert_eq!(authored_grammar_slug(6), None);
        assert_eq!(authored_grammar_region_from_slug("unknown"), None);
        assert_eq!(authored_grammar_region_from_slug("../intro"), None);
    }
}

#[derive(Clone, Copy)]
pub struct GrammarPoint {
    pub label: &'static str,
    pub explanation: &'static str,
    pub example: &'static str,
    pub translation: &'static str,
}

pub struct GrammarCourse {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub points: &'static [GrammarPoint],
}

pub const INTRO: GrammarCourse = GrammarCourse {
    id: "grammar-1",
    title: "Basic Sentence Structure",
    description: "Build simple statements, negatives and yes/no questions in Mandarin.",
    points: &[
        GrammarPoint {
            label: "Subject + predicate",
            explanation: "A straightforward statement usually introduces the subject before what is said about it.",
            example: "我学习中文。",
            translation: "I study Chinese.",
        },
        GrammarPoint {
            label: "是 for identity",
            explanation: "Use 是 between a subject and a noun phrase that identifies it; do not automatically put 是 before an adjective.",
            example: "她是老师。",
            translation: "She is a teacher.",
        },
        GrammarPoint {
            label: "不 before a verb",
            explanation: "For a simple present or habitual negative, 不 normally appears before the verb.",
            example: "我不喝茶。",
            translation: "I do not drink tea.",
        },
        GrammarPoint {
            label: "吗 for yes/no questions",
            explanation: "Add 吗 to the end of an ordinary statement to form a neutral yes/no question.",
            example: "你喜欢咖啡吗？",
            translation: "Do you like coffee?",
        },
    ],
};

pub const TIME_PLACE: GrammarCourse = GrammarCourse {
    id: "grammar-2",
    title: "Time, Place & Everyday Actions",
    description: "Place time and locations naturally in Mandarin statements and questions.",
    points: &[
        GrammarPoint {
            label: "Time before an action",
            explanation: "A time expression often comes after the subject and before the verb. It may also appear at the start for emphasis.",
            example: "我今天学习中文。",
            translation: "I study Chinese today.",
        },
        GrammarPoint {
            label: "在 + place + verb",
            explanation: "Use 在 before a place to say where an action takes place. The location phrase normally comes before the verb.",
            example: "我在学校学习。",
            translation: "I study at school.",
        },
        GrammarPoint {
            label: "Time, then location",
            explanation: "When both occur before the action, a common order is subject + time + location + verb.",
            example: "我今天在学校学习。",
            translation: "I study at school today.",
        },
        GrammarPoint {
            label: "去 + destination",
            explanation: "Use 去 before a destination to express going there; 在 indicates where an action happens, not movement toward that destination.",
            example: "我明天去学校。",
            translation: "I am going to school tomorrow.",
        },
        GrammarPoint {
            label: "Ask where with 哪里",
            explanation: "For asking where an action happens, put 哪里 after 在 in the location slot, before the main verb.",
            example: "你在哪里学习？",
            translation: "Where do you study?",
        },
    ],
};

pub const POSSESSION_QUANTITY: GrammarCourse = GrammarCourse {
    id: "grammar-3",
    title: "Possession, Existence & Quantities",
    description: "Express what someone has, what exists, and ask about counts using classifiers.",
    points: &[
        GrammarPoint {
            label: "有 for possession",
            explanation: "Use 有 after the subject to say that someone has something. Do not add 是 between the subject and 有.",
            example: "我有一本书。",
            translation: "I have one book.",
        },
        GrammarPoint {
            label: "没有 for absence",
            explanation: "Negate 有 with 没有, not 不有. A negative possession sentence does not require an additional 是.",
            example: "我没有电脑。",
            translation: "I do not have a computer.",
        },
        GrammarPoint {
            label: "Number + classifier + noun",
            explanation: "When counting many ordinary nouns, place a classifier between the number and noun: 一本书 or 三个苹果. Choose the classifier suited to the noun.",
            example: "我有三个苹果。",
            translation: "I have three apples.",
        },
        GrammarPoint {
            label: "几 for smaller expected counts",
            explanation: "几 commonly asks for a relatively small number. Keep the appropriate classifier after 几 and before the noun.",
            example: "你有几本书？",
            translation: "How many books do you have?",
        },
        GrammarPoint {
            label: "多少 for open-ended quantities",
            explanation: "多少 asks how many or how much without assuming a small quantity. A classifier can follow 多少 when counting nouns.",
            example: "你有多少个苹果？",
            translation: "How many apples do you have?",
        },
    ],
};

pub const DESCRIPTIONS: GrammarCourse = GrammarCourse {
    id: "grammar-4",
    title: "Descriptions & Comparisons",
    description: "Describe everyday qualities, negate descriptions, express excess and compare two things.",
    points: &[
        GrammarPoint {
            label: "Adjectives as predicates",
            explanation: "Mandarin descriptive adjectives can function as predicates without adding 是. Do not insert 是 directly before 高 or 忙 in ordinary descriptions.",
            example: "他很忙。",
            translation: "He is busy.",
        },
        GrammarPoint {
            label: "Degree with 很",
            explanation: "很 commonly appears before a descriptive adjective. In an unmarked statement it does not always indicate an emphatic English 'very'.",
            example: "这件衣服很漂亮。",
            translation: "This item of clothing is pretty.",
        },
        GrammarPoint {
            label: "Negate an adjective with 不",
            explanation: "Place 不 before the adjective, rather than using 没有 to negate a present-state description.",
            example: "今天不冷。",
            translation: "It is not cold today.",
        },
        GrammarPoint {
            label: "Too ... with 太...了",
            explanation: "Use 太 before the adjective and often 了 after it to express an excessive or strong degree; tone can vary by context.",
            example: "这杯茶太热了。",
            translation: "This cup of tea is too hot.",
        },
        GrammarPoint {
            label: "Compare with 比",
            explanation: "In a basic comparative, put 比 and the comparison target before the adjective: A + 比 + B + adjective.",
            example: "今天比昨天冷。",
            translation: "Today is colder than yesterday.",
        },
    ],
};

pub const WANTS_ABILITIES: GrammarCourse = GrammarCourse {
    id: "grammar-5",
    title: "Wants, Abilities & Polite Requests",
    description: "Express intentions, wants, learned abilities, practical possibility, and permission.",
    points: &[
        GrammarPoint {
            label: "想 + verb for intentions",
            explanation: "想 before an action can express wanting or intending to do it. Keep the action after 想.",
            example: "我想学中文。",
            translation: "I want to learn Chinese.",
        },
        GrammarPoint {
            label: "要 + noun for wants",
            explanation: "要 before a noun can express wanting that item. Context and tone matter: 要 can sound direct, so polite requests may add 请 or other softening language.",
            example: "我要一杯水。",
            translation: "I want a glass of water.",
        },
        GrammarPoint {
            label: "会 for learned skills",
            explanation: "会 before a verb expresses an ability learned through experience, such as speaking a language. It can have other meanings in other contexts.",
            example: "她会说中文。",
            translation: "She can speak Chinese.",
        },
        GrammarPoint {
            label: "能 for practical ability",
            explanation: "能 can describe whether circumstances or physical conditions allow an action. It is not always interchangeable with 会.",
            example: "我今天不能去。",
            translation: "I cannot go today.",
        },
        GrammarPoint {
            label: "可以 for permission",
            explanation: "可以 often expresses permission or acceptability. In a yes/no question, 可以 asks whether something is allowed.",
            example: "我可以坐这里吗？",
            translation: "May I sit here?",
        },
    ],
};

pub const WANTS_ABILITIES_CHECKS: [GuidedQuestion; 5] = [
    GuidedQuestion {
        prompt: "Which says 'I want to learn Chinese'?",
        choices: ["我学想中文。", "我想学中文。", "我会想中文。"],
        correct: 1,
        rationale: "想 comes before the intended action 学中文.",
    },
    GuidedQuestion {
        prompt: "Which asks for a glass of water directly?",
        choices: ["我要一杯水。", "我一杯要水。", "我是一杯水。"],
        correct: 0,
        rationale: "要 comes before the wanted noun phrase 一杯水. Politeness depends on context.",
    },
    GuidedQuestion {
        prompt: "Which says she has learned to speak Chinese?",
        choices: ["她中文会说。", "她说会中文。", "她会说中文。"],
        correct: 2,
        rationale: "会 precedes the learned skill 说中文.",
    },
    GuidedQuestion {
        prompt: "Which says 'I cannot go today'?",
        choices: ["我今天不能去。", "我今天去不能。", "我不今天能去。"],
        correct: 0,
        rationale: "不能 is placed before 去, after 今天.",
    },
    GuidedQuestion {
        prompt: "Which asks permission to sit here?",
        choices: [
            "我坐这里可以吗在哪里？",
            "我可以坐这里吗？",
            "我这里吗可以坐？",
        ],
        correct: 1,
        rationale: "可以 precedes 坐, and the question particle 吗 comes at the end.",
    },
];

pub const POSSESSION_QUANTITY_CHECKS: [GuidedQuestion; 5] = [
    GuidedQuestion {
        prompt: "Which says ‘I have one book’?",
        choices: ["我是一本书。", "我有一本书。", "我一本有书。"],
        correct: 1,
        rationale: "有 indicates possession; 一本 classifies the book.",
    },
    GuidedQuestion {
        prompt: "Which is the correct negative of ‘I have a computer’?",
        choices: ["我不有电脑。", "我没有电脑。", "我不是有电脑。"],
        correct: 1,
        rationale: "Use 没有 to negate possession.",
    },
    GuidedQuestion {
        prompt: "Which says ‘I have three apples’?",
        choices: ["我有三个苹果。", "我有三本苹果。", "我三个有苹果。"],
        correct: 0,
        rationale: "三个苹果 uses the general classifier 个 with the count.",
    },
    GuidedQuestion {
        prompt: "Which asks ‘How many books do you have?’",
        choices: ["你有几书本？", "你几本有书？", "你有几本书？"],
        correct: 2,
        rationale: "几本书 keeps 本 between the quantity question and 书.",
    },
    GuidedQuestion {
        prompt: "Which asks for an unspecified number of apples?",
        choices: ["你有多少个苹果？", "你有多少本苹果？", "你多少有苹果个？"],
        correct: 0,
        rationale: "多少个苹果 uses an open-ended number question and a suitable classifier.",
    },
];

pub struct GuidedQuestion {
    pub prompt: &'static str,
    pub choices: [&'static str; 3],
    pub correct: usize,
    pub rationale: &'static str,
}

pub const INTRO_CHECKS: [GuidedQuestion; 4] = [
    GuidedQuestion {
        prompt: "Which means ‘She is a teacher’?",
        choices: ["她是老师。", "她不老师。", "她吗老师。"],
        correct: 0,
        rationale: "是 links the subject to the noun phrase 老师.",
    },
    GuidedQuestion {
        prompt: "Which means ‘I do not drink tea’?",
        choices: ["我喝不茶。", "我不喝茶。", "我吗喝茶。"],
        correct: 1,
        rationale: "不 precedes the verb 喝.",
    },
    GuidedQuestion {
        prompt: "Which is a yes/no question?",
        choices: ["你喜欢咖啡不。", "你吗喜欢咖啡。", "你喜欢咖啡吗？"],
        correct: 2,
        rationale: "吗 is placed at the end of the statement.",
    },
    GuidedQuestion {
        prompt: "Which has the usual subject–verb–object order?",
        choices: ["我学习中文。", "中文我学习。", "学习我中文。"],
        correct: 0,
        rationale: "我 is the subject, 学习 the verb, and 中文 the object.",
    },
];

pub const TIME_PLACE_CHECKS: [GuidedQuestion; 5] = [
    GuidedQuestion {
        prompt: "Which says ‘I study Chinese today’?",
        choices: ["我今天学习中文。", "我学习今天中文。", "今天中文我学习。"],
        correct: 0,
        rationale: "今天 goes before the verb 学习.",
    },
    GuidedQuestion {
        prompt: "Which says ‘I study at school’?",
        choices: ["我学习在学校。", "我在学校学习。", "我去学校学习在。"],
        correct: 1,
        rationale: "在学校 is the place where studying happens, so it precedes 学习.",
    },
    GuidedQuestion {
        prompt: "Choose the usual order for ‘I study at school today’.",
        choices: [
            "我学校今天在学习。",
            "我学习今天在学校。",
            "我今天在学校学习。",
        ],
        correct: 2,
        rationale: "Subject + time + location + action is a useful default pattern.",
    },
    GuidedQuestion {
        prompt: "Which means ‘I am going to school tomorrow’?",
        choices: ["我明天去学校。", "我明天在学校。", "我明天学校去在。"],
        correct: 0,
        rationale: "去 expresses going to a destination; 在 describes being or acting at a place.",
    },
    GuidedQuestion {
        prompt: "Which asks ‘Where do you study’?",
        choices: ["你学习在哪里？", "你在哪里学习？", "哪里你学习在？"],
        correct: 1,
        rationale: "For the taught pattern, 在哪里 takes the location slot before 学习.",
    },
];

pub const DESCRIPTIONS_CHECKS: [GuidedQuestion; 5] = [
    GuidedQuestion {
        prompt: "Choose 'He is busy.'",
        choices: ["他是忙。", "他很忙。", "他有忙。"],
        correct: 1,
        rationale: "很忙 describes his state; an ordinary adjective predicate does not require 是.",
    },
    GuidedQuestion {
        prompt: "Which describes pretty clothing?",
        choices: ["这件衣服很漂亮。", "这件衣服是漂亮。", "这件衣服漂亮很。"],
        correct: 0,
        rationale: "很 comes before 漂亮 in this ordinary description.",
    },
    GuidedQuestion {
        prompt: "Choose 'It is not cold today.'",
        choices: ["今天没有冷。", "今天是冷不。", "今天不冷。"],
        correct: 2,
        rationale: "不 directly negates the adjective 冷.",
    },
    GuidedQuestion {
        prompt: "Choose 'This tea is too hot.'",
        choices: ["这杯茶了太热。", "这杯茶太热了。", "这杯茶太了热。"],
        correct: 1,
        rationale: "太热了 frames a strong or excessive degree.",
    },
    GuidedQuestion {
        prompt: "Choose 'Today is colder than yesterday.'",
        choices: ["今天比昨天冷。", "今天冷比昨天。", "昨天比今天冷。"],
        correct: 0,
        rationale: "今天 is the compared subject; 比昨天 precedes 冷.",
    },
];

pub fn authored_grammar_region(
    order: usize,
) -> Option<(&'static GrammarCourse, &'static [GuidedQuestion])> {
    match order {
        1 => Some((&INTRO, &INTRO_CHECKS)),
        2 => Some((&TIME_PLACE, &TIME_PLACE_CHECKS)),
        3 => Some((&POSSESSION_QUANTITY, &POSSESSION_QUANTITY_CHECKS)),
        4 => Some((&DESCRIPTIONS, &DESCRIPTIONS_CHECKS)),
        5 => Some((&WANTS_ABILITIES, &WANTS_ABILITIES_CHECKS)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn introductory_course_is_authored_not_placeholder() {
        assert_eq!(INTRO.points.len(), 4);
        assert_eq!(INTRO_CHECKS.len(), 4);
        assert_eq!(TIME_PLACE.points.len(), 5);
        assert_eq!(TIME_PLACE_CHECKS.len(), 5);
        assert_eq!(authored_grammar_region(2).unwrap().0.id, "grammar-2");
        assert_eq!(authored_grammar_region(3).unwrap().0.id, "grammar-3");
        assert_eq!(POSSESSION_QUANTITY.points.len(), 5);
        assert_eq!(POSSESSION_QUANTITY_CHECKS.len(), 5);
        assert_eq!(authored_grammar_region(4).unwrap().0.id, "grammar-4");
        assert_eq!(authored_grammar_region(5).unwrap().0.id, "grammar-5");
        assert!(authored_grammar_region(6).is_none());
        for question in INTRO_CHECKS
            .into_iter()
            .chain(TIME_PLACE_CHECKS)
            .chain(POSSESSION_QUANTITY_CHECKS)
            .chain(DESCRIPTIONS_CHECKS)
            .chain(WANTS_ABILITIES_CHECKS)
        {
            assert!(question.correct < 3);
            assert!(question.choices.iter().all(|option| !option.is_empty()));
            let mut unique = std::collections::BTreeSet::new();
            for choice in question.choices {
                assert!(unique.insert(choice));
            }
        }
    }
}
