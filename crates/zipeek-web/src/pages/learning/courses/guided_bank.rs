#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuidedKind {
    SentenceBuilder,
    SentenceTransformation,
}

#[derive(Clone, Copy, Debug)]
pub struct GuidedActivity {
    pub id: &'static str,
    pub kind: GuidedKind,
    pub title: &'static str,
    pub instruction: &'static str,
    pub source: &'static str,
    pub target_pattern: &'static str,
    pub bank: &'static [&'static str],
    pub presented_order: &'static [usize],
    pub accepted_orders: &'static [&'static [usize]],
    pub explanation: &'static str,
    pub incorrect_feedback: &'static str,
}

pub static ACTIVITIES: [GuidedActivity; 6] = [
    GuidedActivity {
        id: "subject-verb-object",
        kind: GuidedKind::SentenceBuilder,
        title: "Put the subject first",
        instruction: "Build ‘I study Chinese’ in its usual Mandarin order.",
        source: "I study Chinese.",
        target_pattern: "Subject + verb + object",
        bank: &["中文", "我", "学习", "不"],
        presented_order: &[0, 3, 2, 1],
        accepted_orders: &[&[1, 2, 0]],
        explanation: "我 is the subject, 学习 is the verb, and 中文 is the object.",
        incorrect_feedback: "Start with who is doing the action, then the action, then its object.",
    },
    GuidedActivity {
        id: "identity-with-shi",
        kind: GuidedKind::SentenceBuilder,
        title: "Identify someone with 是",
        instruction: "Build ‘She is a teacher’ using 是.",
        source: "She is a teacher.",
        target_pattern: "Subject + 是 + noun",
        bank: &["老师", "是", "她", "不"],
        presented_order: &[3, 0, 2, 1],
        accepted_orders: &[&[2, 1, 0]],
        explanation: "是 links 她 to the identity 老师; it is not a general marker before every predicate.",
        incorrect_feedback: "Place 是 between the person and the noun identifying her.",
    },
    GuidedActivity {
        id: "negative-with-bu",
        kind: GuidedKind::SentenceTransformation,
        title: "Make a sentence negative",
        instruction: "Rewrite the sentence to say ‘I do not drink tea’.",
        source: "我喝茶。",
        target_pattern: "不 before the verb",
        bank: &["喝", "茶", "不", "我", "吗"],
        presented_order: &[1, 4, 0, 3, 2],
        accepted_orders: &[&[3, 2, 0, 1]],
        explanation: "不 belongs immediately before 喝, so the object 茶 stays after the verb.",
        incorrect_feedback: "The negator 不 goes before 喝, not after it.",
    },
    GuidedActivity {
        id: "yes-no-question",
        kind: GuidedKind::SentenceTransformation,
        title: "Ask a yes/no question",
        instruction: "Turn the statement into ‘Do you like coffee?’",
        source: "你喜欢咖啡。",
        target_pattern: "Statement + 吗",
        bank: &["吗", "咖啡", "喜欢", "你", "不"],
        presented_order: &[1, 4, 0, 3, 2],
        accepted_orders: &[&[3, 2, 1, 0]],
        explanation: "The original statement order stays the same; 吗 is added at the end.",
        incorrect_feedback: "Keep 你喜欢咖啡 together, then put 吗 at the end.",
    },
    GuidedActivity {
        id: "identity-negation",
        kind: GuidedKind::SentenceTransformation,
        title: "Negate an identity",
        instruction: "Rewrite ‘She is a teacher’ as ‘She is not a teacher’.",
        source: "她是老师。",
        target_pattern: "Subject + 不是 + noun",
        bank: &["老师", "不", "她", "是", "吗"],
        presented_order: &[3, 0, 4, 1, 2],
        accepted_orders: &[&[2, 1, 3, 0]],
        explanation: "不 comes directly before 是 to form 不是.",
        incorrect_feedback: "Put 不 before 是; the noun phrase comes after 是.",
    },
    GuidedActivity {
        id: "yes-no-check",
        kind: GuidedKind::SentenceBuilder,
        title: "Build a second question",
        instruction: "Build ‘Do you study Chinese?’",
        source: "Do you study Chinese?",
        target_pattern: "Subject + verb + object + 吗",
        bank: &["学习", "中文", "吗", "你", "不"],
        presented_order: &[1, 4, 0, 2, 3],
        accepted_orders: &[&[3, 0, 1, 2]],
        explanation: "Mandarin can form a neutral yes/no question by adding 吗 to the end of a statement.",
        incorrect_feedback: "Start with 你学习中文, then add the question particle 吗.",
    },
];

pub static TIME_PLACE_ACTIVITIES: [GuidedActivity; 6] = [
    GuidedActivity {
        id: "today-study",
        kind: GuidedKind::SentenceBuilder,
        title: "Put today before the verb",
        instruction: "Build ‘I study Chinese today’.",
        source: "I study Chinese today.",
        target_pattern: "Subject + time + verb + object",
        bank: &["中文", "今天", "学习", "我", "在"],
        presented_order: &[4, 2, 0, 3, 1],
        accepted_orders: &[&[3, 1, 2, 0]],
        explanation: "今天 follows 我 and comes before 学习.",
        incorrect_feedback: "Start with 我, then 今天 before the action 学习中文.",
    },
    GuidedActivity {
        id: "school-study",
        kind: GuidedKind::SentenceBuilder,
        title: "Say where you study",
        instruction: "Build ‘I study at school’.",
        source: "I study at school.",
        target_pattern: "Subject + 在 + place + verb",
        bank: &["学习", "学校", "我", "去", "在"],
        presented_order: &[1, 3, 0, 4, 2],
        accepted_orders: &[&[2, 4, 1, 0]],
        explanation: "在学校 identifies where studying happens.",
        incorrect_feedback: "在学校 should come before 学习.",
    },
    GuidedActivity {
        id: "time-and-place",
        kind: GuidedKind::SentenceTransformation,
        title: "Add today to a location sentence",
        instruction: "Transform the sentence to say ‘I study at school today’.",
        source: "我在学校学习。",
        target_pattern: "Subject + time + location + verb",
        bank: &["学校", "今天", "在", "我", "学习", "去"],
        presented_order: &[5, 0, 3, 4, 2, 1],
        accepted_orders: &[&[3, 1, 2, 0, 4]],
        explanation: "今天 comes before 在学校 and the action 学习.",
        incorrect_feedback: "Use 我今天在学校学习 as the taught order.",
    },
    GuidedActivity {
        id: "going-to-school",
        kind: GuidedKind::SentenceBuilder,
        title: "Go somewhere tomorrow",
        instruction: "Build ‘I will go to school tomorrow’.",
        source: "I will go to school tomorrow.",
        target_pattern: "Subject + time + 去 + destination",
        bank: &["学校", "去", "在", "明天", "我"],
        presented_order: &[2, 0, 4, 1, 3],
        accepted_orders: &[&[4, 3, 1, 0]],
        explanation: "去学校 expresses movement toward school, not a static location.",
        incorrect_feedback: "Use 去 before 学校 to indicate the destination.",
    },
    GuidedActivity {
        id: "where-question",
        kind: GuidedKind::SentenceTransformation,
        title: "Ask where the action happens",
        instruction: "Turn ‘You study at school’ into ‘Where do you study?’",
        source: "你在学校学习。",
        target_pattern: "Subject + 在哪里 + verb",
        bank: &["哪里", "学习", "你", "学校", "在"],
        presented_order: &[3, 0, 4, 1, 2],
        accepted_orders: &[&[2, 4, 0, 1]],
        explanation: "在哪里 fills the location slot where 在学校 appeared.",
        incorrect_feedback: "Replace 学校 with 哪里 after 在 and before 学习.",
    },
    GuidedActivity {
        id: "time-place-review",
        kind: GuidedKind::SentenceTransformation,
        title: "Move an action to tomorrow",
        instruction: "Change ‘I study at school today’ to ‘I study at school tomorrow’.",
        source: "我今天在学校学习。",
        target_pattern: "Subject + tomorrow + place + verb",
        bank: &["学习", "在", "今天", "明天", "学校", "我"],
        presented_order: &[4, 2, 0, 5, 3, 1],
        accepted_orders: &[&[5, 3, 1, 4, 0]],
        explanation: "Replace 今天 with 明天; 在学校 remains before 学习.",
        incorrect_feedback: "Keep the location phrase 在学校 before the verb.",
    },
];

pub static POSSESSION_QUANTITY_ACTIVITIES: [GuidedActivity; 6] = [
    GuidedActivity {
        id: "one-book",
        kind: GuidedKind::SentenceBuilder,
        title: "Describe one book",
        instruction: "Build ‘I have one book’.",
        source: "I have one book.",
        target_pattern: "Subject + 有 + number + classifier + noun",
        bank: &["我", "有", "一", "本", "书", "是"],
        presented_order: &[5, 4, 2, 0, 3, 1],
        accepted_orders: &[&[0, 1, 2, 3, 4]],
        explanation: "一本书 has the classifier 本 between the number and 书.",
        incorrect_feedback: "Use 我有, then 一本书; do not add 是.",
    },
    GuidedActivity {
        id: "no-computer",
        kind: GuidedKind::SentenceTransformation,
        title: "Negate possession",
        instruction: "Change ‘I have a computer’ to ‘I do not have a computer’.",
        source: "我有电脑。",
        target_pattern: "Subject + 没有 + noun",
        bank: &["电脑", "有", "我", "没有"],
        presented_order: &[0, 1, 3, 2],
        accepted_orders: &[&[2, 3, 0]],
        explanation: "没有 negates 有 directly.",
        incorrect_feedback: "Use 我没有电脑 rather than 不有.",
    },
    GuidedActivity {
        id: "three-apples",
        kind: GuidedKind::SentenceBuilder,
        title: "Count apples",
        instruction: "Build ‘I have three apples’.",
        source: "I have three apples.",
        target_pattern: "有 + number + 个 + noun",
        bank: &["苹果", "三", "本", "有", "个", "我"],
        presented_order: &[2, 0, 5, 1, 4, 3],
        accepted_orders: &[&[5, 3, 1, 4, 0]],
        explanation: "三个苹果 places 个 between 三 and 苹果.",
        incorrect_feedback: "Put 个, not 本, after 三.",
    },
    GuidedActivity {
        id: "how-many-books",
        kind: GuidedKind::SentenceTransformation,
        title: "Ask how many books",
        instruction: "Change ‘You have books’ into ‘How many books do you have?’.",
        source: "你有书。",
        target_pattern: "Subject + 有 + 几 + classifier + noun",
        bank: &["书", "几", "本", "你", "有", "吗"],
        presented_order: &[0, 5, 3, 2, 4, 1],
        accepted_orders: &[&[3, 4, 1, 2, 0]],
        explanation: "Use 几本书 to ask the number of books.",
        incorrect_feedback: "Put 几 before 本, and 本 before 书.",
    },
    GuidedActivity {
        id: "two-people",
        kind: GuidedKind::SentenceBuilder,
        title: "Count people",
        instruction: "Build ‘There are two people here’.",
        source: "There are two people here.",
        target_pattern: "Place + 有 + number + classifier + noun",
        bank: &["人", "两", "个", "这里", "有", "是"],
        presented_order: &[5, 2, 0, 3, 1, 4],
        accepted_orders: &[&[3, 4, 1, 2, 0]],
        explanation: "这里有 describes existence; 两个人 counts two people.",
        incorrect_feedback: "Start with 这里有, then 两个人.",
    },
    GuidedActivity {
        id: "how-many-apples",
        kind: GuidedKind::SentenceTransformation,
        title: "Ask an open-ended quantity",
        instruction: "Change ‘You have apples’ into ‘How many apples do you have?’.",
        source: "你有苹果。",
        target_pattern: "Subject + 有 + 多少 + classifier + noun",
        bank: &["个", "有", "苹果", "多少", "你", "几"],
        presented_order: &[2, 5, 1, 0, 4, 3],
        accepted_orders: &[&[4, 1, 3, 0, 2]],
        explanation: "多少个苹果 asks about a quantity without presuming it is small.",
        incorrect_feedback: "Use 多少 before 个苹果.",
    },
];

pub static DESCRIPTIONS_ACTIVITIES: [GuidedActivity; 6] = [
    GuidedActivity {
        id: "describe-busy",
        kind: GuidedKind::SentenceBuilder,
        title: "Describe someone",
        instruction: "Build 'He is busy.'",
        source: "He is busy.",
        target_pattern: "Subject + 很 + adjective",
        bank: &["忙", "他", "很", "是"],
        presented_order: &[3, 0, 2, 1],
        accepted_orders: &[&[1, 2, 0]],
        explanation: "很忙 is an adjective predicate; 是 is unnecessary.",
        incorrect_feedback: "Put 很 before 忙, without 是.",
    },
    GuidedActivity {
        id: "describe-clothing",
        kind: GuidedKind::SentenceBuilder,
        title: "Describe clothing",
        instruction: "Build 'This item of clothing is pretty.'",
        source: "This item of clothing is pretty.",
        target_pattern: "Noun phrase + 很 + adjective",
        bank: &["漂亮", "这件衣服", "是", "很"],
        presented_order: &[2, 0, 3, 1],
        accepted_orders: &[&[1, 3, 0]],
        explanation: "很漂亮 follows the subject 这件衣服.",
        incorrect_feedback: "Put 很 directly before 漂亮.",
    },
    GuidedActivity {
        id: "negate-cold",
        kind: GuidedKind::SentenceTransformation,
        title: "Negate a description",
        instruction: "Change 'Today is cold' to 'Today is not cold'.",
        source: "今天很冷。",
        target_pattern: "Time + 不 + adjective",
        bank: &["冷", "今天", "不", "没有"],
        presented_order: &[3, 0, 2, 1],
        accepted_orders: &[&[1, 2, 0]],
        explanation: "不冷 negates a current adjective description.",
        incorrect_feedback: "Use 不 before 冷 rather than 没有.",
    },
    GuidedActivity {
        id: "too-hot-tea",
        kind: GuidedKind::SentenceTransformation,
        title: "Express excessive heat",
        instruction: "Change 'The tea is hot' to 'This tea is too hot'.",
        source: "这杯茶很热。",
        target_pattern: "Subject + 太 + adjective + 了",
        bank: &["热", "了", "这杯茶", "太", "很"],
        presented_order: &[1, 4, 0, 2, 3],
        accepted_orders: &[&[2, 3, 0, 1]],
        explanation: "太热了 expresses excessive or emphatic heat.",
        incorrect_feedback: "Place 太 before 热 and 了 at the end.",
    },
    GuidedActivity {
        id: "compare-weather",
        kind: GuidedKind::SentenceBuilder,
        title: "Compare two days",
        instruction: "Build 'Today is colder than yesterday'.",
        source: "Today is colder than yesterday.",
        target_pattern: "A + 比 + B + adjective",
        bank: &["昨天", "冷", "今天", "比", "很"],
        presented_order: &[4, 1, 0, 3, 2],
        accepted_orders: &[&[2, 3, 0, 1]],
        explanation: "今天比昨天冷 compares today's temperature to yesterday's.",
        incorrect_feedback: "Put 比昨天 before the adjective 冷.",
    },
    GuidedActivity {
        id: "compare-reverse",
        kind: GuidedKind::SentenceTransformation,
        title: "Reverse a comparison",
        instruction: "Change 'Today is colder than yesterday' into 'Yesterday is colder than today'.",
        source: "今天比昨天冷。",
        target_pattern: "A + 比 + B + adjective",
        bank: &["今天", "比", "昨天", "冷", "是"],
        presented_order: &[4, 0, 3, 2, 1],
        accepted_orders: &[&[2, 1, 0, 3]],
        explanation: "Changing the comparison subject reverses the meaning.",
        incorrect_feedback: "Start with 昨天 then 比今天冷.",
    },
];

pub static WANTS_ABILITIES_ACTIVITIES: [GuidedActivity; 6] = [
    GuidedActivity {
        id: "want-to-study",
        kind: GuidedKind::SentenceBuilder,
        title: "Say what you intend",
        instruction: "Build 'I want to learn Chinese.'",
        source: "I want to learn Chinese.",
        target_pattern: "Subject + 想 + action",
        bank: &["中文", "想", "学", "我", "会"],
        presented_order: &[4, 0, 2, 3, 1],
        accepted_orders: &[&[3, 1, 2, 0]],
        explanation: "我想学中文 places 想 before the action 学中文.",
        incorrect_feedback: "Start with 我想 and then 学中文.",
    },
    GuidedActivity {
        id: "want-water",
        kind: GuidedKind::SentenceBuilder,
        title: "Ask for water",
        instruction: "Build 'I want a glass of water.'",
        source: "I want a glass of water.",
        target_pattern: "Subject + 要 + quantity + noun",
        bank: &["水", "我要", "一杯", "是"],
        presented_order: &[3, 0, 2, 1],
        accepted_orders: &[&[1, 2, 0]],
        explanation: "我要一杯水 uses 要 before the noun phrase 一杯水.",
        incorrect_feedback: "Place 我要 before 一杯水.",
    },
    GuidedActivity {
        id: "can-speak",
        kind: GuidedKind::SentenceBuilder,
        title: "Describe a learned ability",
        instruction: "Build 'She can speak Chinese.'",
        source: "She can speak Chinese.",
        target_pattern: "Subject + 会 + learned skill",
        bank: &["中文", "她", "说", "会", "要"],
        presented_order: &[4, 2, 0, 3, 1],
        accepted_orders: &[&[1, 3, 2, 0]],
        explanation: "会说中文 expresses a learned language skill.",
        incorrect_feedback: "Place 会 immediately before 说中文.",
    },
    GuidedActivity {
        id: "cannot-go",
        kind: GuidedKind::SentenceTransformation,
        title: "Express inability today",
        instruction: "Change 'I can go today' to 'I cannot go today'.",
        source: "我今天能去。",
        target_pattern: "Time + 不能 + action",
        bank: &["能", "去", "我", "今天", "不能"],
        presented_order: &[1, 4, 0, 3, 2],
        accepted_orders: &[&[2, 3, 4, 1]],
        explanation: "不能去 follows the time expression 今天.",
        incorrect_feedback: "Use 不能 rather than 能 before 去.",
    },
    GuidedActivity {
        id: "sit-here-permission",
        kind: GuidedKind::SentenceBuilder,
        title: "Ask to sit here",
        instruction: "Build 'May I sit here?'",
        source: "May I sit here?",
        target_pattern: "Subject + 可以 + verb phrase + 吗",
        bank: &["坐", "吗", "我", "这里", "可以", "想"],
        presented_order: &[5, 0, 3, 1, 4, 2],
        accepted_orders: &[&[2, 4, 0, 3, 1]],
        explanation: "可以 asks permission, and 吗 turns the request into a question.",
        incorrect_feedback: "Start with 我可以坐这里 and add 吗 at the end.",
    },
    GuidedActivity {
        id: "intend-tomorrow",
        kind: GuidedKind::SentenceTransformation,
        title: "Express a new intention",
        instruction: "Change 'I study today' to 'I want to study tomorrow'.",
        source: "我今天学习。",
        target_pattern: "Subject + time + 想 + action",
        bank: &["明天", "想", "学习", "我", "今天"],
        presented_order: &[4, 2, 0, 3, 1],
        accepted_orders: &[&[3, 0, 1, 2]],
        explanation: "我明天想学习 puts 明天 before 想学习.",
        incorrect_feedback: "Replace 今天 with 明天, and put 想 before 学习.",
    },
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GuidedCourse {
    #[default]
    Intro,
    TimePlace,
    PossessionQuantity,
    Descriptions,
    WantsAbilities,
}

impl GuidedCourse {
    pub fn region(self) -> usize {
        match self {
            Self::Intro => 1,
            Self::TimePlace => 2,
            Self::PossessionQuantity => 3,
            Self::Descriptions => 4,
            Self::WantsAbilities => 5,
        }
    }

    pub fn from_region(region: usize) -> Option<Self> {
        match region {
            1 => Some(Self::Intro),
            2 => Some(Self::TimePlace),
            3 => Some(Self::PossessionQuantity),
            4 => Some(Self::Descriptions),
            5 => Some(Self::WantsAbilities),
            _ => None,
        }
    }

    pub fn activities(self) -> &'static [GuidedActivity] {
        match self {
            Self::Intro => &ACTIVITIES,
            Self::TimePlace => &TIME_PLACE_ACTIVITIES,
            Self::PossessionQuantity => &POSSESSION_QUANTITY_ACTIVITIES,
            Self::Descriptions => &DESCRIPTIONS_ACTIVITIES,
            Self::WantsAbilities => &WANTS_ABILITIES_ACTIVITIES,
        }
    }
}

pub fn assembled(activity: &GuidedActivity, positions: &[usize]) -> Option<String> {
    if positions.iter().any(|&id| id >= activity.bank.len()) {
        return None;
    }
    let mut visited = std::collections::BTreeSet::new();
    if positions.iter().any(|id| !visited.insert(*id)) {
        return None;
    }
    Some(
        positions
            .iter()
            .map(|&id| activity.bank[id])
            .collect::<String>(),
    )
}

pub fn accepts(activity: &GuidedActivity, positions: &[usize]) -> bool {
    activity.accepted_orders.contains(&positions)
}

pub fn reference_answer(activity: &GuidedActivity) -> String {
    assembled(activity, activity.accepted_orders[0]).unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuidedSession {
    pub index: usize,
    pub selected: Vec<usize>,
    pub checked: bool,
    pub correct: usize,
    pub course: GuidedCourse,
}

impl Default for GuidedSession {
    fn default() -> Self {
        Self {
            index: 0,
            selected: Vec::new(),
            checked: false,
            correct: 0,
            course: GuidedCourse::Intro,
        }
    }
}

impl GuidedSession {
    pub fn for_course(course: GuidedCourse) -> Self {
        Self {
            course,
            ..Self::default()
        }
    }

    pub fn finished(&self) -> bool {
        self.index >= self.course.activities().len()
    }

    pub fn select(&mut self, id: usize) {
        if self.finished() || self.checked {
            return;
        }
        if id < self.course.activities()[self.index].bank.len() && !self.selected.contains(&id) {
            self.selected.push(id);
        }
    }

    pub fn remove(&mut self, position: usize) {
        if !self.checked && position < self.selected.len() {
            self.selected.remove(position);
        }
    }

    pub fn undo(&mut self) {
        if !self.checked {
            self.selected.pop();
        }
    }

    pub fn reset_selection(&mut self) {
        if !self.checked {
            self.selected.clear();
        }
    }

    pub fn check(&mut self) {
        if !self.finished() && !self.checked && !self.selected.is_empty() {
            self.checked = true;
            if accepts(&self.course.activities()[self.index], &self.selected) {
                self.correct += 1;
            }
        }
    }

    pub fn advance(&mut self) {
        if self.checked && !self.finished() {
            self.index += 1;
            self.checked = false;
            self.selected.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_banks_are_consistent_and_have_distinct_distractors() {
        let mut ids = std::collections::BTreeSet::new();
        for activity in ACTIVITIES
            .into_iter()
            .chain(TIME_PLACE_ACTIVITIES)
            .chain(POSSESSION_QUANTITY_ACTIVITIES)
            .chain(DESCRIPTIONS_ACTIVITIES)
        {
            assert!(ids.insert(activity.id));
            assert!(activity.bank.len() > activity.accepted_orders[0].len());
            assert_eq!(activity.presented_order.len(), activity.bank.len());
            let presented = activity
                .presented_order
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(presented, (0..activity.bank.len()).collect());
            let words = activity
                .bank
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(words.len(), activity.bank.len());
            for order in activity.accepted_orders {
                assert!(!order.is_empty());
                assert!(assembled(&activity, order).is_some());
                assert!(accepts(&activity, order));
            }
            assert!(!activity.explanation.is_empty());
        }
    }

    #[test]
    fn second_course_completes_without_awarding_mastery() {
        let mut session = GuidedSession::for_course(GuidedCourse::TimePlace);
        for activity in TIME_PLACE_ACTIVITIES {
            for &tile in activity.accepted_orders[0] {
                session.select(tile);
            }
            session.check();
            session.advance();
        }
        assert!(session.finished());
        assert_eq!(session.correct, TIME_PLACE_ACTIVITIES.len());
    }

    #[test]
    fn third_course_completes_without_awarding_mastery() {
        let mut session = GuidedSession::for_course(GuidedCourse::PossessionQuantity);
        for activity in POSSESSION_QUANTITY_ACTIVITIES {
            for &tile in activity.accepted_orders[0] {
                session.select(tile);
            }
            session.check();
            session.advance();
        }
        assert!(session.finished());
        assert_eq!(session.correct, POSSESSION_QUANTITY_ACTIVITIES.len());
    }

    #[test]
    fn fourth_course_completes_without_awarding_mastery() {
        let mut session = GuidedSession::for_course(GuidedCourse::Descriptions);
        for activity in DESCRIPTIONS_ACTIVITIES {
            for &tile in activity.accepted_orders[0] {
                session.select(tile);
            }
            session.check();
            session.advance();
        }
        assert!(session.finished());
        assert_eq!(session.correct, DESCRIPTIONS_ACTIVITIES.len());
    }

    #[test]
    fn fifth_course_completes_without_awarding_mastery() {
        let mut session = GuidedSession::for_course(GuidedCourse::WantsAbilities);
        for activity in WANTS_ABILITIES_ACTIVITIES {
            for &tile in activity.accepted_orders[0] {
                session.select(tile);
            }
            session.check();
            session.advance();
        }
        assert!(session.finished());
        assert_eq!(session.correct, WANTS_ABILITIES_ACTIVITIES.len());
    }

    #[test]
    fn duplicate_and_out_of_range_tiles_are_rejected() {
        let activity = &ACTIVITIES[0];
        assert!(assembled(activity, &[1, 1]).is_none());
        assert!(assembled(activity, &[19]).is_none());
        assert!(!accepts(activity, &[1, 1]));
    }

    #[test]
    fn session_guards_checking_editing_and_advancing() {
        let mut session = GuidedSession::default();
        session.check();
        session.advance();
        assert_eq!(session.index, 0);
        for &tile in ACTIVITIES[0].accepted_orders[0] {
            session.select(tile);
        }
        session.select(ACTIVITIES[0].accepted_orders[0][0]);
        assert_eq!(session.selected.len(), 3);
        session.check();
        session.check();
        assert_eq!(session.correct, 1);
        session.remove(0);
        session.select(3);
        assert_eq!(session.selected.len(), 3);
        session.advance();
        assert_eq!(session.index, 1);
        assert!(session.selected.is_empty());
    }

    #[test]
    fn incorrect_answers_do_not_award_correct_or_block_next_activity() {
        let mut session = GuidedSession::default();
        session.select(3);
        session.check();
        assert_eq!(session.correct, 0);
        session.advance();
        assert_eq!(session.index, 1);
    }

    #[test]
    fn full_guided_flow_finishes_without_mastery_tracking() {
        let mut session = GuidedSession::default();
        for activity in ACTIVITIES {
            for &tile in activity.accepted_orders[0] {
                session.select(tile);
            }
            session.check();
            session.advance();
        }
        assert!(session.finished());
        assert_eq!(session.correct, ACTIVITIES.len());
    }
}
