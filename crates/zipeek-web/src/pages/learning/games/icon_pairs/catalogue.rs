use std::collections::BTreeMap;
use yew_icons::IconData;

#[derive(Clone, Copy)]
pub(super) struct IconWord {
    pub icon: IconData,
    pub icon_key: &'static str,
    pub hanzi: &'static str,
}

struct IconAssociation {
    key: &'static str,
    icon: IconData,
    words: &'static [&'static str],
}

const ASSOCIATIONS: &[IconAssociation] = &[
    IconAssociation {
        key: "lucide:sun",
        icon: IconData::LUCIDE_SUN,
        words: &["太阳", "晴天"],
    },
    IconAssociation {
        key: "lucide:moon",
        icon: IconData::LUCIDE_MOON,
        words: &["晚上", "月亮"],
    },
    IconAssociation {
        key: "lucide:cloud",
        icon: IconData::LUCIDE_CLOUD,
        words: &["天气", "阴天"],
    },
    IconAssociation {
        key: "lucide:home",
        icon: IconData::LUCIDE_HOME,
        words: &["家", "房子"],
    },
    IconAssociation {
        key: "lucide:book-open",
        icon: IconData::LUCIDE_BOOK_OPEN,
        words: &["学习", "读书"],
    },
    IconAssociation {
        key: "lucide:clock",
        icon: IconData::LUCIDE_CLOCK,
        words: &["时间", "小时"],
    },
    IconAssociation {
        key: "lucide:heart",
        icon: IconData::LUCIDE_HEART,
        words: &["喜欢", "爱"],
    },
    IconAssociation {
        key: "lucide:gift",
        icon: IconData::LUCIDE_GIFT,
        words: &["礼物", "生日"],
    },
    IconAssociation {
        key: "lucide:coffee",
        icon: IconData::LUCIDE_COFFEE,
        words: &["喝茶", "咖啡"],
    },
    IconAssociation {
        key: "lucide:music",
        icon: IconData::LUCIDE_MUSIC,
        words: &["音乐", "唱歌"],
    },
    IconAssociation {
        key: "lucide:map",
        icon: IconData::LUCIDE_MAP,
        words: &["旅行", "地图"],
    },
    IconAssociation {
        key: "lucide:camera",
        icon: IconData::LUCIDE_CAMERA,
        words: &["照片", "拍照"],
    },
    IconAssociation {
        key: "lucide:ear",
        icon: IconData::LUCIDE_EAR,
        words: &["听", "耳朵"],
    },
    IconAssociation {
        key: "lucide:pencil",
        icon: IconData::LUCIDE_PENCIL,
        words: &["写字", "笔", "学习"],
    },
    IconAssociation {
        key: "lucide:message-circle",
        icon: IconData::LUCIDE_MESSAGE_CIRCLE,
        words: &["说话", "聊天"],
    },
    IconAssociation {
        key: "lucide:users",
        icon: IconData::LUCIDE_USERS,
        words: &["朋友", "大家"],
    },
    IconAssociation {
        key: "lucide:utensils",
        icon: IconData::LUCIDE_UTENSILS,
        words: &["吃饭", "餐厅"],
    },
    IconAssociation {
        key: "lucide:shopping-bag",
        icon: IconData::LUCIDE_SHOPPING_BAG,
        words: &["买东西", "商店"],
    },
    IconAssociation {
        key: "lucide:car",
        icon: IconData::LUCIDE_CAR,
        words: &["开车", "汽车"],
    },
    IconAssociation {
        key: "lucide:train",
        icon: IconData::LUCIDE_TRAIN,
        words: &["火车", "车站"],
    },
    IconAssociation {
        key: "lucide:tree-pine",
        icon: IconData::LUCIDE_TREE_PINE,
        words: &["森林", "树"],
    },
    IconAssociation {
        key: "lucide:mountain",
        icon: IconData::LUCIDE_MOUNTAIN,
        words: &["山", "爬山"],
    },
    IconAssociation {
        key: "lucide:star",
        icon: IconData::LUCIDE_STAR,
        words: &["星星", "夜空"],
    },
    IconAssociation {
        key: "lucide:key",
        icon: IconData::LUCIDE_KEY,
        words: &["钥匙", "开门"],
    },
];

pub(super) fn catalogue() -> BTreeMap<&'static str, Vec<IconData>> {
    let mut words: BTreeMap<&'static str, Vec<IconData>> = BTreeMap::new();
    for association in ASSOCIATIONS {
        for &hanzi in association.words {
            words.entry(hanzi).or_default().push(association.icon);
        }
    }
    words
}

pub(super) fn all_words() -> Vec<IconWord> {
    ASSOCIATIONS
        .iter()
        .flat_map(|association| {
            association.words.iter().map(move |&hanzi| IconWord {
                icon: association.icon,
                icon_key: association.key,
                hanzi,
            })
        })
        .collect()
}

pub(super) fn icon_count() -> usize {
    ASSOCIATIONS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mappings_are_indexed_by_simplified_hanzi() {
        let words = catalogue();
        assert!(words.len() >= 48);
        assert_eq!(words.get("学习").map(Vec::len), Some(2));
        assert_eq!(words.get("太阳").map(Vec::len), Some(1));
        assert_eq!(
            all_words().len(),
            words.values().map(Vec::len).sum::<usize>()
        );
    }

    #[test]
    fn source_icon_identifiers_are_unique() {
        let identifiers = ASSOCIATIONS
            .iter()
            .map(|entry| entry.key)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(identifiers.len(), ASSOCIATIONS.len());
        assert!(ASSOCIATIONS.iter().all(|entry| !entry.words.is_empty()));
    }
}
