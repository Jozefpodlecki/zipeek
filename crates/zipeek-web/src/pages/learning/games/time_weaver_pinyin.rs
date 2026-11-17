pub(super) fn reading(text: &str) -> String {
    if text == "一刻" {
        return "yí kè".to_owned();
    }
    text.chars()
        .map(|character| match character {
            '零' => "líng",
            '一' => "yī",
            '二' => "èr",
            '三' => "sān",
            '四' => "sì",
            '五' => "wǔ",
            '六' => "liù",
            '七' => "qī",
            '八' => "bā",
            '九' => "jiǔ",
            '十' => "shí",
            '两' => "liǎng",
            '年' => "nián",
            '月' => "yuè",
            '日' => "rì",
            '星' => "xīng",
            '期' => "qī",
            '点' => "diǎn",
            '分' => "fēn",
            '刻' => "kè",
            '半' => "bàn",
            '凌' => "líng",
            '晨' => "chén",
            '早' => "zǎo",
            '上' => "shàng",
            '午' => "wǔ",
            '中' => "zhōng",
            '下' => "xià",
            '晚' => "wǎn",
            _ => "",
        })
        .filter(|syllable| !syllable.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::reading;

    #[test]
    fn date_weekday_and_time_tiles_have_readings() {
        assert_eq!(reading("二十一"), "èr shí yī");
        assert_eq!(reading("星期三"), "xīng qī sān");
        assert_eq!(reading("下午两点半"), "xià wǔ liǎng diǎn bàn");
    }
}
