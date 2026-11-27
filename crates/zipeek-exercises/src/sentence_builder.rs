mod state;

pub use state::{
    BuilderAction, BuilderState, DragSource, DropTarget, MoveDirection, TileId, Verdict,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SentencePrompt {
    pub meaning: &'static str,
    pub tiles: &'static [&'static str],
    pub accepted_orders: &'static [&'static [usize]],
    pub explanation: &'static str,
}

pub const PROMPTS: &[SentencePrompt] = &[
    SentencePrompt {
        meaning: "I study Chinese every day.",
        tiles: &["中文", "我", "每天", "学习"],
        accepted_orders: &[&[1, 2, 3, 0]],
        explanation: "每天 is placed before the verb 学习; the object 中文 follows the verb.",
    },
    SentencePrompt {
        meaning: "Tomorrow I will go to school.",
        tiles: &["学校", "我", "去", "明天"],
        accepted_orders: &[&[1, 3, 2, 0], &[3, 1, 2, 0]],
        explanation: "The time expression 明天 may appear before or after the subject.",
    },
    SentencePrompt {
        meaning: "We drink tea at home.",
        tiles: &["喝茶", "在家", "我们"],
        accepted_orders: &[&[2, 1, 0]],
        explanation: "The location phrase 在家 precedes the verb phrase 喝茶.",
    },
    SentencePrompt {
        meaning: "She reads books at the library every weekend.",
        tiles: &["图书馆", "她", "每个周末", "在", "看书"],
        accepted_orders: &[&[1, 2, 3, 0, 4]],
        explanation: "每个周末 is a time phrase; 在图书馆 gives the location.",
    },
    SentencePrompt {
        meaning: "My younger brother likes eating apples.",
        tiles: &["喜欢", "我弟弟", "苹果", "吃"],
        accepted_orders: &[&[1, 0, 3, 2]],
        explanation: "The subject comes before 喜欢 and the verb phrase 吃苹果 follows.",
    },
    SentencePrompt {
        meaning: "We will eat dinner together tonight.",
        tiles: &["一起", "晚饭", "今天晚上", "我们", "吃"],
        accepted_orders: &[&[3, 2, 0, 4, 1]],
        explanation: "今天晚上 indicates time; 一起 modifies the verb.",
    },
    SentencePrompt {
        meaning: "The teacher is explaining the grammar.",
        tiles: &["老师", "语法", "正在", "解释"],
        accepted_orders: &[&[0, 2, 3, 1]],
        explanation: "正在 marks a progressive action before the verb.",
    },
    SentencePrompt {
        meaning: "He bought two tickets yesterday.",
        tiles: &["两张票", "昨天", "他", "买了"],
        accepted_orders: &[&[2, 1, 3, 0]],
        explanation: "The classifier 张 is used with tickets.",
    },
    SentencePrompt {
        meaning: "Because it is raining, we are staying home.",
        tiles: &["下雨了", "因为", "我们", "所以", "在家"],
        accepted_orders: &[&[1, 0, 3, 2, 4]],
        explanation: "因为 introduces the reason; 所以 introduces the result.",
    },
    SentencePrompt {
        meaning: "This restaurant is cheaper than that one.",
        tiles: &["比", "便宜", "这家饭店", "那家"],
        accepted_orders: &[&[2, 0, 3, 1]],
        explanation: "比 marks the compared object before the adjective.",
    },
    SentencePrompt {
        meaning: "Please speak a little more slowly.",
        tiles: &["一点儿", "说", "请", "慢"],
        accepted_orders: &[&[2, 1, 3, 0]],
        explanation: "请 is polite, and 慢一点儿 describes a slower pace.",
    },
    SentencePrompt {
        meaning: "I want to drink a cup of hot tea.",
        tiles: &["一杯", "我", "热茶", "想喝"],
        accepted_orders: &[&[1, 3, 0, 2]],
        explanation: "一杯 counts tea; 热 modifies 茶.",
    },
    SentencePrompt {
        meaning: "After class we will play football.",
        tiles: &["我们", "踢足球", "下课以后", "去"],
        accepted_orders: &[&[2, 0, 3, 1]],
        explanation: "下课以后 places the action after class.",
    },
    SentencePrompt {
        meaning: "The book you lent me is very interesting.",
        tiles: &["很有意思", "你借给我的", "这本书"],
        accepted_orders: &[&[1, 2, 0]],
        explanation: "The modifier 你借给我的 precedes 这本书.",
    },
    SentencePrompt {
        meaning: "She has already finished her homework.",
        tiles: &["做完", "作业", "已经", "她", "了"],
        accepted_orders: &[&[3, 2, 0, 1, 4]],
        explanation: "已经 combines with the result complement 做完.",
    },
    SentencePrompt {
        meaning: "Tomorrow the weather may be cold.",
        tiles: &["很冷", "可能", "天气", "明天"],
        accepted_orders: &[&[3, 2, 1, 0]],
        explanation: "可能 expresses uncertainty before the adjective.",
    },
    SentencePrompt {
        meaning: "Can you help me find the train station?",
        tiles: &["帮我", "你能", "火车站", "找到", "吗"],
        accepted_orders: &[&[1, 0, 3, 2, 4]],
        explanation: "能 introduces ability; 吗 makes the question.",
    },
    SentencePrompt {
        meaning: "I am learning Chinese while listening to music.",
        tiles: &["音乐", "一边", "我", "学中文", "听", "一边"],
        accepted_orders: &[&[2, 1, 4, 0, 5, 3]],
        explanation: "一边…一边… combines simultaneous actions.",
    },
    SentencePrompt {
        meaning: "The dog is sleeping under the table.",
        tiles: &["桌子下面", "睡觉", "小狗", "在"],
        accepted_orders: &[&[2, 3, 0, 1]],
        explanation: "在 introduces the location of an action.",
    },
    SentencePrompt {
        meaning: "She went to the supermarket to buy vegetables.",
        tiles: &["去", "她", "买菜", "超市"],
        accepted_orders: &[&[1, 0, 3, 2]],
        explanation: "The destination precedes the purpose 买菜.",
    },
    SentencePrompt {
        meaning: "If you have time, call me.",
        tiles: &["给我打电话", "有时间", "如果你", "就"],
        accepted_orders: &[&[2, 1, 3, 0]],
        explanation: "如果…就… links the condition and outcome.",
    },
    SentencePrompt {
        meaning: "I have never been to Beijing.",
        tiles: &["北京", "我", "没", "去过"],
        accepted_orders: &[&[1, 2, 3, 0]],
        explanation: "没…过 negates a past experience.",
    },
    SentencePrompt {
        meaning: "The children are playing happily in the park.",
        tiles: &["玩得很开心", "孩子们", "在公园"],
        accepted_orders: &[&[1, 2, 0]],
        explanation: "得 introduces a manner complement after the verb.",
    },
];

pub fn move_tile(order: &[usize], from: usize, to: usize) -> Option<Vec<usize>> {
    if from >= order.len() || to >= order.len() {
        return None;
    }
    let mut reordered = order.to_vec();
    let tile = reordered.remove(from);
    reordered.insert(to, tile);
    Some(reordered)
}

pub fn prompt_index_for_session(session_seed: u64, ordinal: usize) -> usize {
    (session_seed as usize).wrapping_add(ordinal) % PROMPTS.len()
}

pub fn validate(prompt: &SentencePrompt) -> bool {
    let len = prompt.tiles.len();
    len >= 2
        && !prompt.meaning.trim().is_empty()
        && !prompt.explanation.trim().is_empty()
        && prompt.tiles.iter().all(|tile| !tile.trim().is_empty())
        && !prompt.accepted_orders.is_empty()
        && prompt
            .accepted_orders
            .iter()
            .enumerate()
            .all(|(index, order)| {
                if prompt.accepted_orders[..index].contains(order) {
                    return false;
                }
                order.len() == len && {
                    let mut seen = vec![false; len];
                    order.iter().all(|&id| {
                        if id >= len || seen[id] {
                            return false;
                        }
                        seen[id] = true;
                        true
                    })
                }
            })
}

pub fn grade(prompt: &SentencePrompt, selected_ids: &[usize]) -> bool {
    validate(prompt)
        && prompt
            .accepted_orders
            .iter()
            .any(|order| *order == selected_ids)
}

pub fn sentence(prompt: &SentencePrompt, selected_ids: &[usize]) -> Option<String> {
    if !validate(prompt) || selected_ids.len() != prompt.tiles.len() {
        return None;
    }
    let mut used = vec![false; prompt.tiles.len()];
    let mut output = String::new();
    for &id in selected_ids {
        if id >= prompt.tiles.len() || used[id] {
            return None;
        }
        used[id] = true;
        output.push_str(prompt.tiles[id]);
    }
    Some(output)
}

pub fn shuffled_tile_ids(prompt: &SentencePrompt, seed: u64) -> Vec<usize> {
    let mut indices: Vec<_> = (0..prompt.tiles.len()).collect();
    let mut state = seed ^ 0x9e3779b97f4a7c15;
    for i in (1..indices.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let j = (state.wrapping_mul(0x2545f4914f6cdd1d) % (i as u64 + 1)) as usize;
        indices.swap(i, j);
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_uses_distinct_prompts_for_twenty_steps() {
        assert!(PROMPTS.len() >= 20);
        for seed in [0, 7, u64::MAX] {
            let indices = (0..20)
                .map(|step| prompt_index_for_session(seed, step))
                .collect::<std::collections::HashSet<_>>();
            assert_eq!(indices.len(), 20);
        }
    }

    #[test]
    fn selected_tiles_can_be_reordered_safely() {
        assert_eq!(move_tile(&[2, 1, 0], 0, 2), Some(vec![1, 0, 2]));
        assert_eq!(move_tile(&[2, 1, 0], 2, 0), Some(vec![0, 2, 1]));
        assert_eq!(move_tile(&[2, 1, 0], 3, 0), None);
    }

    #[test]
    fn corpus_and_reviewed_alternatives_are_valid() {
        assert!(PROMPTS.iter().all(validate));
        assert!(grade(&PROMPTS[1], &[1, 3, 2, 0]));
        assert!(grade(&PROMPTS[1], &[3, 1, 2, 0]));
        assert!(!grade(&PROMPTS[1], &[1, 2, 3, 0]));
        assert!(!grade(&PROMPTS[1], &[1, 3, 2, 2]));
    }

    #[test]
    fn duplicate_answer_orders_are_rejected_as_invalid_content() {
        let invalid = SentencePrompt {
            accepted_orders: &[&[0, 1], &[0, 1]],
            tiles: &["我", "去"],
            ..PROMPTS[0]
        };
        assert!(!validate(&invalid));
        assert!(!grade(&invalid, &[0, 1]));
    }

    #[test]
    fn stable_ids_handle_repeated_text_without_reusing_tiles() {
        let prompt = SentencePrompt {
            meaning: "Test repetition",
            tiles: &["我", "和", "我"],
            accepted_orders: &[&[0, 1, 2]],
            explanation: "Test example",
        };
        assert!(grade(&prompt, &[0, 1, 2]));
        assert!(!grade(&prompt, &[0, 1, 0]));
        assert_eq!(sentence(&prompt, &[0, 1, 2]).as_deref(), Some("我和我"));
        assert_eq!(sentence(&prompt, &[0, 0, 2]), None);
    }

    #[test]
    fn shuffle_is_seeded_and_preserves_every_id() {
        for prompt in PROMPTS {
            let shuffled = shuffled_tile_ids(prompt, 123);
            assert_eq!(shuffled, shuffled_tile_ids(prompt, 123));
            let mut sorted = shuffled;
            sorted.sort_unstable();
            assert_eq!(sorted, (0..prompt.tiles.len()).collect::<Vec<_>>());
        }
    }

    #[test]
    fn malformed_review_content_cannot_render_an_answer() {
        let malformed = SentencePrompt {
            accepted_orders: &[],
            ..PROMPTS[0]
        };
        assert!(!validate(&malformed));
        assert_eq!(sentence(&malformed, &[1, 2, 3, 0]), None);
        assert_eq!(sentence(&PROMPTS[0], &[1, 1, 3, 0]), None);
        assert_eq!(sentence(&PROMPTS[0], &[1, 2, 3, 50]), None);
    }
}
