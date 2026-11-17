use super::assessment_quality::validate_item_quality;
use super::curriculum_registry::{GRAMMAR_REGISTRY, registry_entry};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssessmentItem {
    pub id: &'static str,
    pub point_id: &'static str,
    pub prompt: &'static str,
    pub choices: [&'static str; 3],
    pub correct: usize,
    pub rationale: &'static str,
}

pub const ITEMS: [AssessmentItem; 24] = [
    AssessmentItem {
        id: "a01-order",
        point_id: "g01-subject-predicate",
        prompt: "Choose the normal sentence: ‘She studies Chinese.’",
        choices: ["中文学习她。", "她学习中文。", "学习她中文。"],
        correct: 1,
        rationale: "The usual statement order here is subject 她, verb 学习, object 中文.",
    },
    AssessmentItem {
        id: "a01-identity",
        point_id: "g01-shi-identity",
        prompt: "Choose ‘He is a student.’",
        choices: ["他学生是。", "他有学生。", "他是学生。"],
        correct: 2,
        rationale: "是 joins a subject and a noun phrase describing identity.",
    },
    AssessmentItem {
        id: "a01-negation",
        point_id: "g01-bu-negation",
        prompt: "Choose ‘She does not drink coffee.’",
        choices: ["她不喝咖啡。", "她喝不咖啡。", "她没是喝咖啡。"],
        correct: 0,
        rationale: "Place 不 before the verb 喝 for this habitual negation.",
    },
    AssessmentItem {
        id: "a01-question",
        point_id: "g01-ma-question",
        prompt: "Choose ‘Do you study Chinese?’",
        choices: ["吗你学习中文？", "你学习吗中文？", "你学习中文吗？"],
        correct: 2,
        rationale: "The sentence-final particle 吗 turns this statement into a yes/no question.",
    },
    AssessmentItem {
        id: "a02-time",
        point_id: "g02-time-placement",
        prompt: "Choose ‘I study tomorrow.’",
        choices: ["我明天学习。", "我学习明天。", "明天我学习吗。"],
        correct: 0,
        rationale: "明天 occurs before the action; an added 吗 changes the meaning.",
    },
    AssessmentItem {
        id: "a02-location",
        point_id: "g02-zai-location",
        prompt: "Choose ‘She reads at home.’",
        choices: ["她看书在家。", "她在家看书。", "她是家看书。"],
        correct: 1,
        rationale: "在家 gives the location before the verb phrase 看书.",
    },
    AssessmentItem {
        id: "a02-both",
        point_id: "g02-time-location",
        prompt: "Choose ‘I study at home today.’",
        choices: ["我在今天家学习。", "我学习今天在家。", "我今天在家学习。"],
        correct: 2,
        rationale: "今天 (time) and 在家 (place) come before 学习.",
    },
    AssessmentItem {
        id: "a02-go",
        point_id: "g02-qu-destination",
        prompt: "Choose ‘He goes to school tomorrow.’",
        choices: ["他明天去学校。", "他明天在学校。", "他去明天学校。"],
        correct: 0,
        rationale: "去学校 expresses going to school; 在学校 would describe being at school.",
    },
    AssessmentItem {
        id: "a02-where",
        point_id: "g02-nali-question",
        prompt: "Choose ‘Where do you read?’",
        choices: ["你哪看里书？", "你在哪里看书？", "你在看哪里书？"],
        correct: 1,
        rationale: "在哪里 asks for the location, followed by the action 看书.",
    },
    AssessmentItem {
        id: "a03-have",
        point_id: "g03-you-possession",
        prompt: "Choose ‘She has a dog.’",
        choices: ["她是一个狗。", "她一个有狗。", "她有一只狗。"],
        correct: 2,
        rationale: "有 expresses possession; 只 is an appropriate classifier for a dog.",
    },
    AssessmentItem {
        id: "a03-not-have",
        point_id: "g03-meiyou-negation",
        prompt: "Choose ‘I do not have a car.’",
        choices: ["我没有车。", "我不有车。", "我不是有车。"],
        correct: 0,
        rationale: "没有 negates having something; 不有 is not the standard construction.",
    },
    AssessmentItem {
        id: "a03-classifier",
        point_id: "g03-classifiers",
        prompt: "Choose ‘two books.’",
        choices: ["两只书", "两本书", "两张书"],
        correct: 1,
        rationale: "本 is the normal classifier for bound books.",
    },
    AssessmentItem {
        id: "a03-ji",
        point_id: "g03-ji-count",
        prompt: "Choose ‘How many cats do you have?’ (small expected count)",
        choices: ["你有几只猫？", "你有几本猫？", "你有几猫只？"],
        correct: 0,
        rationale: "几 + appropriate classifier 只 + 猫 asks about a relatively small number.",
    },
    AssessmentItem {
        id: "a03-duoshao",
        point_id: "g03-duoshao-quantity",
        prompt: "Choose ‘How many people are there?’",
        choices: ["有几本个人？", "有多少本个人？", "有多少人？"],
        correct: 2,
        rationale: "多少人 asks an open-ended quantity of people without a classifier.",
    },
    AssessmentItem {
        id: "a04-adjective",
        point_id: "g04-adjective-predicate",
        prompt: "Choose 'She is busy.'",
        choices: ["她是忙。", "她有忙。", "她很忙。"],
        correct: 2,
        rationale: "忙 is an adjective predicate, commonly preceded by 很 without 是.",
    },
    AssessmentItem {
        id: "a04-hen",
        point_id: "g04-hen-degree",
        prompt: "Choose 'This item of clothing is pretty.'",
        choices: ["这件衣服很漂亮。", "这件衣服漂亮很。", "这件衣服是很漂亮。"],
        correct: 0,
        rationale: "很 directly precedes 漂亮 in the ordinary adjective predicate.",
    },
    AssessmentItem {
        id: "a04-bu",
        point_id: "g04-bu-adjective",
        prompt: "Choose 'Today is not cold.'",
        choices: ["今天没有冷。", "今天不冷。", "今天冷不。"],
        correct: 1,
        rationale: "不 negates the adjective 冷.",
    },
    AssessmentItem {
        id: "a04-tai",
        point_id: "g04-tai-le",
        prompt: "Choose 'This tea is too hot.'",
        choices: ["这杯茶太了热。", "这杯茶了太热。", "这杯茶太热了。"],
        correct: 2,
        rationale: "太 + adjective + 了 is a common strong-degree pattern.",
    },
    AssessmentItem {
        id: "a04-bi",
        point_id: "g04-bi-comparison",
        prompt: "Choose 'Today is colder than yesterday.'",
        choices: ["昨天比今天冷。", "今天比昨天冷。", "今天冷比昨天。"],
        correct: 1,
        rationale: "今天比昨天冷 puts the target after 比 and before 冷.",
    },
    AssessmentItem {
        id: "a05-xiang",
        point_id: "g05-xiang-intent",
        prompt: "Choose 'I want to learn Chinese.'",
        choices: ["我想学中文。", "我学想中文。", "我想中文学。"],
        correct: 0,
        rationale: "想 is followed by the intended action 学中文.",
    },
    AssessmentItem {
        id: "a05-yao",
        point_id: "g05-yao-want",
        prompt: "Choose 'I want a glass of water.'",
        choices: ["我一杯水要。", "我要一杯水。", "我是一杯水。"],
        correct: 1,
        rationale: "要 precedes 一杯水, the wanted noun phrase.",
    },
    AssessmentItem {
        id: "a05-hui",
        point_id: "g05-hui-learned-ability",
        prompt: "Choose 'She can speak Chinese' (a learned skill).",
        choices: ["她中文会说。", "她说会中文。", "她会说中文。"],
        correct: 2,
        rationale: "会 comes before the learned ability 说中文.",
    },
    AssessmentItem {
        id: "a05-neng",
        point_id: "g05-neng-possibility",
        prompt: "Choose 'I cannot go today.'",
        choices: ["我今天不能去。", "我不能今天去。", "我今天去不能。"],
        correct: 0,
        rationale: "不能 precedes the action 去; 今天 is placed before it.",
    },
    AssessmentItem {
        id: "a05-keyi",
        point_id: "g05-keyi-permission",
        prompt: "Choose the question 'May I sit here?'",
        choices: ["我坐这里吗可以？", "我可以坐这里吗？", "我吗可以坐这里？"],
        correct: 1,
        rationale: "可以 asks permission before 坐; 吗 follows the verb phrase.",
    },
];

pub fn items_for_region(region: usize) -> Vec<&'static AssessmentItem> {
    let Some(entry) = registry_entry(region) else {
        return Vec::new();
    };
    entry
        .point_ids
        .iter()
        .filter_map(|point| ITEMS.iter().find(|item| item.point_id == *point))
        .collect()
}

pub fn validate_assessments() -> Result<(), &'static str> {
    let mut item_ids = BTreeSet::new();
    let mut point_ids = BTreeSet::new();
    let mut prompts = BTreeSet::new();
    for item in ITEMS {
        if item.id.is_empty() || !item_ids.insert(item.id) {
            return Err("duplicate or empty item id");
        }
        if !point_ids.insert(item.point_id) {
            return Err("duplicate assessment point id");
        }
        validate_item_quality(&item)?;
        if !prompts.insert(item.prompt.trim().to_lowercase()) {
            return Err("duplicate assessment prompt");
        }
    }
    for course in GRAMMAR_REGISTRY {
        for point_id in course.point_ids {
            if !point_ids.contains(point_id) {
                return Err("authored grammar point missing assessment");
            }
        }
        if items_for_region(course.region).len() != course.point_ids.len() {
            return Err("course assessment count mismatch");
        }
    }
    if point_ids.len()
        != GRAMMAR_REGISTRY
            .iter()
            .map(|course| course.point_ids.len())
            .sum::<usize>()
    {
        return Err("orphan assessment point");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_assessments_cover_every_point_once() {
        assert_eq!(validate_assessments(), Ok(()));
        assert_eq!(items_for_region(1).len(), 4);
        assert_eq!(items_for_region(2).len(), 5);
        assert_eq!(items_for_region(3).len(), 5);
        assert_eq!(items_for_region(4).len(), 5);
        assert_eq!(items_for_region(5).len(), 5);
        assert!(items_for_region(6).is_empty());
    }
}
