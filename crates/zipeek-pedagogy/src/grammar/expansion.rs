use super::curriculum::{GrammarCourse, GrammarPoint};

pub static EXPANDED_COURSES: [GrammarCourse; 15] = [
    GrammarCourse {
        id: "grammar-6",
        title: "Aspect: completed events",
        description: "Talk about completed events and changes of situation.",
        points: &[
            GrammarPoint {
                label: "了 after a verb",
                explanation: "A verb followed by 了 can present an event as completed, but does not always refer to past time.",
                example: "我买了一本书。",
                translation: "I bought a book.",
            },
            GrammarPoint {
                label: "Sentence-final 了",
                explanation: "Sentence-final 了 frequently signals a new situation or change, rather than just a completed action.",
                example: "下雨了。",
                translation: "It has started raining.",
            },
            GrammarPoint {
                label: "没(有) with completed actions",
                explanation: "Use 没 or 没有, not 不, to negate an event that did not happen.",
                example: "我昨天没去学校。",
                translation: "I did not go to school yesterday.",
            },
            GrammarPoint {
                label: "了 with quantified objects",
                explanation: "A completed action with a quantified object can use verb + 了 + quantity + noun.",
                example: "她喝了两杯水。",
                translation: "She drank two glasses of water.",
            },
            GrammarPoint {
                label: "Already with 已经",
                explanation: "已经 marks an event or state as already established; 了 may also occur when appropriate.",
                example: "我已经吃饭了。",
                translation: "I have already eaten.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-7",
        title: "Experience and ongoing states",
        description: "Differentiate experience, duration and continuing situations.",
        points: &[
            GrammarPoint {
                label: "过 for experience",
                explanation: "Place 过 after a verb to express that someone has had an experience at least once.",
                example: "我去过北京。",
                translation: "I have been to Beijing.",
            },
            GrammarPoint {
                label: "没(有) ... 过",
                explanation: "Negate previous experience with 没有 followed by verb + 过.",
                example: "我没有看过这部电影。",
                translation: "I have never seen this film.",
            },
            GrammarPoint {
                label: "着 for a continuing state",
                explanation: "着 after a verb can describe a continuing state, including an object's position.",
                example: "门开着。",
                translation: "The door is open.",
            },
            GrammarPoint {
                label: "正在 for ongoing actions",
                explanation: "正在 before a verb highlights an activity happening at the reference time.",
                example: "他正在做饭。",
                translation: "He is cooking.",
            },
            GrammarPoint {
                label: "一边 ... 一边 ...",
                explanation: "Use 一边 with two verb phrases to show simultaneous activities.",
                example: "她一边走路一边听音乐。",
                translation: "She listens to music while walking.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-8",
        title: "Duration and frequency",
        description: "Describe how long, how often and when an event occurs.",
        points: &[
            GrammarPoint {
                label: "Duration after the verb",
                explanation: "An action's duration commonly follows the verb, with object placement depending on the construction.",
                example: "我学了两年中文。",
                translation: "I studied Chinese for two years.",
            },
            GrammarPoint {
                label: "Verb repetition with duration",
                explanation: "With some verb-object phrases, repeat the verb before a duration expression.",
                example: "他游泳游了一个小时。",
                translation: "He swam for an hour.",
            },
            GrammarPoint {
                label: "每 ... 都 ...",
                explanation: "每 introduces a recurring time or member, while 都 emphasizes coverage of each instance.",
                example: "我每天都读书。",
                translation: "I read every day.",
            },
            GrammarPoint {
                label: "又 versus 再",
                explanation: "又 often refers to repetition that has happened; 再 usually points to another occurrence later.",
                example: "他昨天又迟到了。",
                translation: "He was late again yesterday.",
            },
            GrammarPoint {
                label: "才 versus 就 in time",
                explanation: "才 and 就 can convey that an event occurs later or earlier than expected.",
                example: "她八点才到。",
                translation: "She did not arrive until eight.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-9",
        title: "Location and existence",
        description: "Describe locations, the presence of things and movement to a place.",
        points: &[
            GrammarPoint {
                label: "Place + 有 + noun",
                explanation: "A location may be the topic before 有 to introduce something present there.",
                example: "桌子上有一本书。",
                translation: "There is a book on the table.",
            },
            GrammarPoint {
                label: "Place + 是 + noun",
                explanation: "Use 是 after a location to identify what occupies that place.",
                example: "前面是图书馆。",
                translation: "The library is ahead.",
            },
            GrammarPoint {
                label: "Locative nouns",
                explanation: "Use 上, 下, 里, 外 and similar locatives after nouns to specify positions.",
                example: "猫在桌子下面。",
                translation: "The cat is under the table.",
            },
            GrammarPoint {
                label: "从 ... 到 ...",
                explanation: "Use 从 and 到 to introduce the beginning and endpoint of a route or time range.",
                example: "我从家走到学校。",
                translation: "I walk from home to school.",
            },
            GrammarPoint {
                label: "离 for distance",
                explanation: "离 introduces the reference point when describing the distance between places.",
                example: "我家离学校很近。",
                translation: "My home is close to the school.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-10",
        title: "Direction and movement",
        description: "Combine motion verbs with direction and destination.",
        points: &[
            GrammarPoint {
                label: "来 versus 去",
                explanation: "来 expresses motion toward the reference point, while 去 expresses motion away.",
                example: "请进来。",
                translation: "Please come in.",
            },
            GrammarPoint {
                label: "上来 and 上去",
                explanation: "上 with 来 or 去 combines upward motion with its direction relative to the speaker.",
                example: "他跑上楼去了。",
                translation: "He ran upstairs away from here.",
            },
            GrammarPoint {
                label: "进来 and 进去",
                explanation: "进来 and 进去 indicate entering, with the motion oriented toward or away from the reference point.",
                example: "她走进教室来了。",
                translation: "She walked into the classroom toward us.",
            },
            GrammarPoint {
                label: "回来 and 回去",
                explanation: "回来 signals returning toward the reference point, while 回去 signals returning elsewhere.",
                example: "你什么时候回来？",
                translation: "When are you coming back?",
            },
            GrammarPoint {
                label: "出来 and 出去",
                explanation: "出来 and 出去 describe movement out of a place toward or away from the reference point.",
                example: "孩子们跑出去了。",
                translation: "The children ran outside.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-11",
        title: "Result complements",
        description: "Express the result of an action using verb complements.",
        points: &[
            GrammarPoint {
                label: "Verb + 完",
                explanation: "完 following a verb expresses finishing the action or its object.",
                example: "我看完这本书了。",
                translation: "I finished reading this book.",
            },
            GrammarPoint {
                label: "Verb + 到",
                explanation: "到 after a verb can express attaining the intended result, such as finding or hearing.",
                example: "我找到钥匙了。",
                translation: "I found the key.",
            },
            GrammarPoint {
                label: "Verb + 懂",
                explanation: "懂 following a perception verb expresses successful comprehension.",
                example: "你听懂了吗？",
                translation: "Did you understand what you heard?",
            },
            GrammarPoint {
                label: "Verb + 好",
                explanation: "好 may describe achieving a satisfactory finished result.",
                example: "饭做好了。",
                translation: "The meal is ready.",
            },
            GrammarPoint {
                label: "Verb + 错",
                explanation: "错 after a verb expresses that an action produced an incorrect result.",
                example: "我写错了名字。",
                translation: "I wrote the name incorrectly.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-12",
        title: "Possibility complements",
        description: "Express whether a result or direction is achievable.",
        points: &[
            GrammarPoint {
                label: "Verb + 得 + result",
                explanation: "得 between a verb and result complements often describes whether a result is possible.",
                example: "这个字我看得懂。",
                translation: "I can understand this character.",
            },
            GrammarPoint {
                label: "Verb + 不 + result",
                explanation: "不 in a potential complement expresses inability to achieve the stated result.",
                example: "我听不懂这个问题。",
                translation: "I cannot understand this question.",
            },
            GrammarPoint {
                label: "做得完 and 做不完",
                explanation: "Use a potential complement with 完 to express whether a task can be completed.",
                example: "今天做得完吗？",
                translation: "Can it be finished today?",
            },
            GrammarPoint {
                label: "看得见 and 看不见",
                explanation: "见 can combine with a perception verb to express whether something is perceptible.",
                example: "我看不见黑板。",
                translation: "I cannot see the blackboard.",
            },
            GrammarPoint {
                label: "进得去 and 进不去",
                explanation: "Directional potential complements describe whether entering or leaving is possible.",
                example: "门太小，车进不去。",
                translation: "The door is too small for the car to enter.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-13",
        title: "Degree and manner complements",
        description: "Explain how an action is performed and to what degree.",
        points: &[
            GrammarPoint {
                label: "Verb + 得 + adjective",
                explanation: "得 after a verb can introduce a description of how the action is performed.",
                example: "他说得很清楚。",
                translation: "He speaks very clearly.",
            },
            GrammarPoint {
                label: "Verb-object with manner",
                explanation: "In verb-object phrases, the verb may be repeated before a 得 manner complement.",
                example: "她唱歌唱得很好。",
                translation: "She sings very well.",
            },
            GrammarPoint {
                label: "Adjective + 极了",
                explanation: "极了 follows an adjective to express a strong degree.",
                example: "今天热极了。",
                translation: "It is extremely hot today.",
            },
            GrammarPoint {
                label: "得多 in comparisons",
                explanation: "得多 after an adjective can intensify a comparative difference.",
                example: "这个房间大得多。",
                translation: "This room is much bigger.",
            },
            GrammarPoint {
                label: "一点儿 for small differences",
                explanation: "一点儿 after an adjective can express a small difference in a comparison.",
                example: "今天比昨天冷一点儿。",
                translation: "Today is a little colder than yesterday.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-14",
        title: "The 把 construction",
        description: "Use 把 to emphasize handling an identifiable object.",
        points: &[
            GrammarPoint {
                label: "Basic 把 sentence",
                explanation: "A basic 把 sentence places a specific object before the verb and normally includes a result or other complement.",
                example: "请把门关上。",
                translation: "Please close the door.",
            },
            GrammarPoint {
                label: "把 + object + 放在",
                explanation: "Use 把 with 放在 to specify where an object is moved or placed.",
                example: "我把书放在桌子上。",
                translation: "I put the book on the table.",
            },
            GrammarPoint {
                label: "把 + object + 给",
                explanation: "把 may introduce an object transferred to a recipient using 给.",
                example: "请把这个给老师。",
                translation: "Please give this to the teacher.",
            },
            GrammarPoint {
                label: "Negative 把 with 没",
                explanation: "Negative markers usually appear before 把, not after the object.",
                example: "我没把作业写完。",
                translation: "I did not finish the homework.",
            },
            GrammarPoint {
                label: "把 with result complement",
                explanation: "A result complement such as 好 or 完 can make the handling outcome explicit.",
                example: "他把房间打扫干净了。",
                translation: "He cleaned the room thoroughly.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-15",
        title: "Passive constructions",
        description: "Describe affected participants and express passive meanings.",
        points: &[
            GrammarPoint {
                label: "被 + agent + verb",
                explanation: "被 can introduce an agent when the subject undergoes an action.",
                example: "我的书被弟弟拿走了。",
                translation: "My younger brother took my book away.",
            },
            GrammarPoint {
                label: "被 without an agent",
                explanation: "The agent may be omitted when unknown or irrelevant.",
                example: "门被打开了。",
                translation: "The door was opened.",
            },
            GrammarPoint {
                label: "让 in passive contexts",
                explanation: "让 can introduce an agent in some colloquial passive-like constructions.",
                example: "他让雨淋湿了。",
                translation: "He got soaked by the rain.",
            },
            GrammarPoint {
                label: "叫 in passive contexts",
                explanation: "叫 may introduce an agent in colloquial passive constructions.",
                example: "杯子叫孩子打破了。",
                translation: "The child broke the cup.",
            },
            GrammarPoint {
                label: "Affected subject without 被",
                explanation: "Some passive meanings are conveyed by context without a passive marker.",
                example: "杯子打破了。",
                translation: "The cup was broken.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-16",
        title: "Noun modification and 的",
        description: "Build more complex noun phrases with modifiers and relative clauses.",
        points: &[
            GrammarPoint {
                label: "Adjective + 的 + noun",
                explanation: "An adjective may be linked to its noun with 的, especially when the modifier is longer or contrastive.",
                example: "我喜欢红色的衣服。",
                translation: "I like red clothes.",
            },
            GrammarPoint {
                label: "Possessor + 的 + noun",
                explanation: "的 links many possessors and possessed nouns.",
                example: "这是老师的书。",
                translation: "This is the teacher's book.",
            },
            GrammarPoint {
                label: "Relative clause + 的 + noun",
                explanation: "Place the relative clause before 的 and the noun it modifies.",
                example: "这是我昨天买的书。",
                translation: "This is the book I bought yesterday.",
            },
            GrammarPoint {
                label: "Verb phrase as modifier",
                explanation: "A verb phrase before 的 can identify someone or something by an action.",
                example: "正在唱歌的人是我妹妹。",
                translation: "The person singing is my younger sister.",
            },
            GrammarPoint {
                label: "Nominalized 的",
                explanation: "的 can stand for an understood noun in context.",
                example: "红色的是我的。",
                translation: "The red one is mine.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-17",
        title: "Topic and emphasis",
        description: "Organize information around topics and focus.",
        points: &[
            GrammarPoint {
                label: "Topic followed by comment",
                explanation: "A known topic may appear before a comment describing it.",
                example: "这本书，我看过。",
                translation: "This book, I have read it.",
            },
            GrammarPoint {
                label: "连 ... 都 ...",
                explanation: "连 highlights a surprising extreme and 都 gives the inclusive focus.",
                example: "连小孩子都知道。",
                translation: "Even children know.",
            },
            GrammarPoint {
                label: "是 ... 的 for past details",
                explanation: "是 ... 的 can focus a detail of a completed situation, such as when or how it happened.",
                example: "我是坐火车来的。",
                translation: "I came by train.",
            },
            GrammarPoint {
                label: "只有 ... 才 ...",
                explanation: "只有 introduces a necessary condition and 才 introduces the resulting possibility.",
                example: "只有努力，才能进步。",
                translation: "Only through effort can one improve.",
            },
            GrammarPoint {
                label: "不仅 ... 还 ...",
                explanation: "不仅 adds one proposition and 还 provides an additional point.",
                example: "他不仅会中文，还会日语。",
                translation: "He knows not only Chinese but also Japanese.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-18",
        title: "Conditionals and cause",
        description: "Connect causes, conditions and consequences.",
        points: &[
            GrammarPoint {
                label: "因为 ... 所以 ...",
                explanation: "因为 introduces a cause, and 所以 introduces its consequence.",
                example: "因为下雨，所以我们没出去。",
                translation: "Because it rained, we did not go out.",
            },
            GrammarPoint {
                label: "如果 ... 就 ...",
                explanation: "如果 introduces a hypothetical condition, while 就 marks the consequence.",
                example: "如果明天下雨，我们就不去。",
                translation: "If it rains tomorrow, we will not go.",
            },
            GrammarPoint {
                label: "要是 ... 就 ...",
                explanation: "要是 provides a conversational alternative for a hypothetical condition.",
                example: "要是你有时间，就来找我。",
                translation: "If you have time, come see me.",
            },
            GrammarPoint {
                label: "只要 ... 就 ...",
                explanation: "只要 introduces a sufficient condition, with 就 marking what follows.",
                example: "只要坚持，就会进步。",
                translation: "As long as you persist, you will improve.",
            },
            GrammarPoint {
                label: "既然 ... 就 ...",
                explanation: "既然 presents an accepted premise before a consequent suggestion or judgment.",
                example: "既然来了，就坐一会儿吧。",
                translation: "Since you are here, stay a while.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-19",
        title: "Contrast and concession",
        description: "Express contrasts and unexpected relationships between clauses.",
        points: &[
            GrammarPoint {
                label: "虽然 ... 但是 ...",
                explanation: "虽然 introduces a concession, and 但是 introduces a contrasting fact.",
                example: "虽然很累，但是她还在学习。",
                translation: "Although tired, she is still studying.",
            },
            GrammarPoint {
                label: "即使 ... 也 ...",
                explanation: "即使 sets up a hypothetical concession, with 也 marking a result that still holds.",
                example: "即使下雨，我也要去。",
                translation: "Even if it rains, I will go.",
            },
            GrammarPoint {
                label: "尽管 ... 还是 ...",
                explanation: "尽管 acknowledges a fact while 还是 maintains the outcome.",
                example: "尽管很难，他还是做到了。",
                translation: "Although it was hard, he still managed it.",
            },
            GrammarPoint {
                label: "反而 for contrary outcomes",
                explanation: "反而 highlights a result opposite to what might be expected.",
                example: "他没有生气，反而笑了。",
                translation: "He did not get angry; instead he smiled.",
            },
            GrammarPoint {
                label: "而 as a contrastive linker",
                explanation: "而 can contrast related clauses in more formal language.",
                example: "他喜欢茶，而我喜欢咖啡。",
                translation: "He likes tea, whereas I like coffee.",
            },
        ],
    },
    GrammarCourse {
        id: "grammar-20",
        title: "Sequence and complex actions",
        description: "Connect events in temporal or logical order.",
        points: &[
            GrammarPoint {
                label: "先 ... 再 ...",
                explanation: "先 introduces an earlier action; 再 introduces the next action.",
                example: "先洗手，再吃饭。",
                translation: "Wash your hands before eating.",
            },
            GrammarPoint {
                label: "一 ... 就 ...",
                explanation: "一 ... 就 ... expresses that one event follows another immediately.",
                example: "我一到家就给你打电话。",
                translation: "I will call you as soon as I get home.",
            },
            GrammarPoint {
                label: "然后 for sequence",
                explanation: "然后 links consecutive actions or steps in a narrative.",
                example: "我吃了饭，然后去散步。",
                translation: "I ate and then went for a walk.",
            },
            GrammarPoint {
                label: "一边 ... 一边 ... in narration",
                explanation: "一边 can link actions occurring simultaneously within a narrative.",
                example: "他一边做饭一边聊天。",
                translation: "He chats while cooking.",
            },
            GrammarPoint {
                label: "直到 ... 才 ...",
                explanation: "直到 ... 才 ... emphasizes that an event did not occur until a given time or condition.",
                example: "直到晚上他才回来。",
                translation: "He did not return until evening.",
            },
        ],
    },
];

pub fn expanded_course(region: usize) -> Option<&'static GrammarCourse> {
    match region {
        6..=20 => EXPANDED_COURSES.get(region - 6),
        21..=100 => super::completed::completed_course(region),
        _ => None,
    }
}

pub fn expanded_course_by_id(id: &str) -> Option<&'static GrammarCourse> {
    let region = id.strip_prefix("grammar-")?.parse::<usize>().ok()?;
    let course = expanded_course(region)?;
    (course.id == id).then_some(course)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_courses_have_distinct_authored_patterns() {
        let mut labels = std::collections::BTreeSet::new();
        assert_eq!(EXPANDED_COURSES.len(), 15);
        assert!(expanded_course_by_id("grammar-100").is_some());
        assert!(expanded_course_by_id("grammar-101").is_none());
        assert!(expanded_course_by_id("grammar-021").is_none());
        for (index, course) in EXPANDED_COURSES.iter().enumerate() {
            assert_eq!(course.id, format!("grammar-{}", index + 6));
            assert_eq!(course.points.len(), 5);
            for point in course.points {
                assert!(!point.explanation.is_empty());
                assert!(
                    point
                        .example
                        .chars()
                        .any(|c| ('\u{3400}'..='\u{9fff}').contains(&c))
                );
                assert!(!point.translation.is_empty());
                assert!(labels.insert((course.id, point.label)));
            }
        }
    }
}
