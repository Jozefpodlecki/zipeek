use zipeek_core::seeded_random::SeededRandom;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChineseDate {
    year: u16,
    month: u8,
    day: u8,
}

impl ChineseDate {
    pub fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        if !(1900..=2099).contains(&year) || !(1..=12).contains(&month) {
            return None;
        }
        (day >= 1 && day <= days_in_month(year, month)).then_some(Self { year, month, day })
    }

    pub fn hanzi(self) -> String {
        format!(
            "{}年{}月{}日",
            self.year_hanzi(),
            chinese_number(self.month),
            chinese_number(self.day)
        )
    }

    pub fn year_hanzi(self) -> String {
        self.year
            .to_string()
            .chars()
            .map(|digit| match digit {
                '0' => '零',
                '1' => '一',
                '2' => '二',
                '3' => '三',
                '4' => '四',
                '5' => '五',
                '6' => '六',
                '7' => '七',
                '8' => '八',
                '9' => '九',
                _ => unreachable!("year contains only decimal digits"),
            })
            .collect()
    }

    pub fn tiles(self) -> Vec<String> {
        let mut tiles: Vec<String> = self
            .year_hanzi()
            .chars()
            .map(|digit| digit.to_string())
            .collect();
        tiles.extend([
            "年".to_owned(),
            chinese_number(self.month),
            "月".to_owned(),
            chinese_number(self.day),
            "日".to_owned(),
        ]);
        tiles
    }

    pub fn western(self) -> String {
        format!("{} {} {}", self.day, english_month(self.month), self.year)
    }

    pub fn weekday_hanzi(self) -> &'static str {
        const WEEKDAYS: [&str; 7] = [
            "星期日",
            "星期一",
            "星期二",
            "星期三",
            "星期四",
            "星期五",
            "星期六",
        ];
        let mut year = i32::from(self.year);
        let month = usize::from(self.month);
        if month < 3 {
            year -= 1;
        }
        let offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let weekday =
            (year + year / 4 - year / 100 + year / 400 + offsets[month - 1] + i32::from(self.day))
                % 7;
        WEEKDAYS[weekday as usize]
    }

    pub fn weekday_english(self) -> &'static str {
        match self.weekday_hanzi() {
            "星期日" => "Sunday",
            "星期一" => "Monday",
            "星期二" => "Tuesday",
            "星期三" => "Wednesday",
            "星期四" => "Thursday",
            "星期五" => "Friday",
            "星期六" => "Saturday",
            _ => unreachable!("weekday is always one of seven values"),
        }
    }

    fn adjacent(self, forward: bool) -> Self {
        let mut year = self.year;
        let mut month = self.month;
        let day;
        if forward {
            if self.day < days_in_month(year, month) {
                day = self.day + 1;
            } else {
                day = 1;
                if month == 12 {
                    month = 1;
                    year += 1;
                } else {
                    month += 1;
                }
            }
        } else if self.day > 1 {
            day = self.day - 1;
        } else {
            if month == 1 {
                month = 12;
                year -= 1;
            } else {
                month -= 1;
            }
            day = days_in_month(year, month);
        }
        Self { year, month, day }
    }
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 31,
    }
}

fn chinese_number(value: u8) -> String {
    const DIGITS: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    match value {
        1..=9 => DIGITS[usize::from(value)].to_owned(),
        10 => "十".to_owned(),
        11..=19 => format!("十{}", DIGITS[usize::from(value % 10)]),
        20 | 30 => format!("{}十", DIGITS[usize::from(value / 10)]),
        21..=29 | 31 => format!(
            "{}十{}",
            DIGITS[usize::from(value / 10)],
            DIGITS[usize::from(value % 10)]
        ),
        _ => unreachable!("month or day out of range"),
    }
}

fn minute_number(value: u8) -> String {
    if value <= 31 {
        return chinese_number(value);
    }
    const DIGITS: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    let tens = usize::from(value / 10);
    let ones = usize::from(value % 10);
    if ones == 0 {
        format!("{}十", DIGITS[tens])
    } else {
        format!("{}十{}", DIGITS[tens], DIGITS[ones])
    }
}

fn english_month(month: u8) -> &'static str {
    [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ][usize::from(month - 1)]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChineseTime {
    hour: u8,
    minute: u8,
}

impl ChineseTime {
    pub fn new(hour: u8, minute: u8) -> Option<Self> {
        (hour < 24 && minute < 60).then_some(Self { hour, minute })
    }

    fn period(self) -> &'static str {
        match self.hour {
            0..=5 => "凌晨",
            6..=8 => "早上",
            9..=11 => "上午",
            12 => "中午",
            13..=17 => "下午",
            _ => "晚上",
        }
    }

    fn hour_hanzi(self) -> &'static str {
        const HOURS: [&str; 13] = [
            "十二", "一", "两", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二",
        ];
        HOURS[usize::from(if self.hour % 12 == 0 {
            12
        } else {
            self.hour % 12
        })]
    }

    fn minute_hanzi(self) -> String {
        match self.minute {
            0 => String::new(),
            15 => "一刻".to_owned(),
            30 => "半".to_owned(),
            45 => "三刻".to_owned(),
            1..=9 => format!("零{}分", chinese_number(self.minute)),
            _ => format!("{}分", minute_number(self.minute)),
        }
    }

    pub fn hanzi(self) -> String {
        format!(
            "{}{}点{}",
            self.period(),
            self.hour_hanzi(),
            self.minute_hanzi()
        )
    }

    pub fn tiles(self) -> Vec<String> {
        let mut tiles = vec![
            self.period().to_owned(),
            self.hour_hanzi().to_owned(),
            "点".to_owned(),
        ];
        match self.minute {
            0 => (),
            15 => tiles.push("一刻".to_owned()),
            30 => tiles.push("半".to_owned()),
            45 => tiles.push("三刻".to_owned()),
            1..=9 => {
                tiles.push("零".to_owned());
                tiles.push(chinese_number(self.minute));
                tiles.push("分".to_owned());
            }
            _ => {
                tiles.push(minute_number(self.minute));
                tiles.push("分".to_owned());
            }
        }
        tiles
    }

    pub fn western(self) -> String {
        let hour = if self.hour % 12 == 0 {
            12
        } else {
            self.hour % 12
        };
        format!(
            "{}:{:02} {}",
            hour,
            self.minute,
            if self.hour < 12 { "AM" } else { "PM" }
        )
    }

    fn with_minute(self, minute: u8) -> Self {
        Self { minute, ..self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateQuestionKind {
    BuildHanzi,
    ChooseHanzi,
    ChooseWeekday,
    BuildTime,
    ChooseTime,
}

impl DateQuestionKind {
    pub fn is_build(self) -> bool {
        matches!(self, Self::BuildHanzi | Self::BuildTime)
    }
    pub fn is_time(self) -> bool {
        matches!(self, Self::BuildTime | Self::ChooseTime)
    }
}

const DEFAULT_QUESTION_COUNT: usize = 15;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Question {
    kind: DateQuestionKind,
    date: ChineseDate,
    clock: ChineseTime,
    correct_option: usize,
    tile_order: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeWeaver {
    questions: Vec<Question>,
    index: usize,
    selected_tiles: Vec<usize>,
    result: Option<bool>,
    chosen_option: Option<usize>,
    score: usize,
}

impl Default for TimeWeaver {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeWeaver {
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    pub fn with_seed(seed: u64) -> Self {
        Self::with_seed_and_count(seed, DEFAULT_QUESTION_COUNT)
    }

    pub fn with_seed_and_count(seed: u64, count: usize) -> Self {
        assert!(matches!(count, 10 | 15 | 20), "Time Weaver supports 10, 15 or 20 questions");
        let mut random = SeededRandom::new(seed);
        let mut kinds = Vec::with_capacity(count);
        for _ in 0..(count / 5) {
            kinds.extend([
                DateQuestionKind::BuildHanzi,
                DateQuestionKind::ChooseHanzi,
                DateQuestionKind::ChooseWeekday,
                DateQuestionKind::BuildTime,
                DateQuestionKind::ChooseTime,
            ]);
        }
        random.shuffle(&mut kinds);
        let mut used_dates = std::collections::BTreeSet::new();
        let mut used_times = std::collections::BTreeSet::new();
        let questions = kinds
            .into_iter()
            .map(|kind| {
                let date = loop {
                    let year = (1900 + random.index(200)) as u16;
                    let month = (1 + random.index(12)) as u8;
                    let day = (1 + random.index(usize::from(days_in_month(year, month)))) as u8;
                    if used_dates.insert((year, month, day)) {
                        break ChineseDate { year, month, day };
                    }
                };
                let clock = loop {
                    let hour = random.index(24) as u8;
                    let minute = random.index(60) as u8;
                    if used_times.insert((hour, minute)) {
                        break ChineseTime { hour, minute };
                    }
                };
                let count = match kind {
                    DateQuestionKind::BuildHanzi => date.tiles().len(),
                    DateQuestionKind::BuildTime => clock.tiles().len(),
                    _ => 0,
                };
                let mut tile_order: Vec<usize> = (0..count).collect();
                random.shuffle(&mut tile_order);
                Question {
                    kind,
                    date,
                    clock,
                    correct_option: random.index(3),
                    tile_order,
                }
            })
            .collect();
        Self {
            questions,
            index: 0,
            selected_tiles: Vec::new(),
            result: None,
            chosen_option: None,
            score: 0,
        }
    }

    pub fn total(&self) -> usize {
        self.questions.len()
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn score(&self) -> usize {
        self.score
    }
    pub fn finished(&self) -> bool {
        self.index == self.total()
    }
    pub fn reviewed(&self) -> Option<bool> {
        self.result
    }
    pub fn chosen_option(&self) -> Option<usize> {
        self.chosen_option
    }
    pub fn submitted_answer(&self) -> Option<String> {
        self.result?;
        if self.question_kind().is_build() {
            let tiles = self.tiles()?;
            Some(
                self.selected_tiles
                    .iter()
                    .map(|&index| tiles[index].as_str())
                    .collect(),
            )
        } else {
            let selected = self.chosen_option?;
            self.options().map(|choices| choices[selected].clone())
        }
    }
    pub fn question_kind(&self) -> DateQuestionKind {
        self.questions
            .get(self.index)
            .map_or(DateQuestionKind::BuildHanzi, |question| question.kind)
    }
    pub fn date(&self) -> Option<ChineseDate> {
        self.questions.get(self.index).map(|question| question.date)
    }
    pub fn clock(&self) -> Option<ChineseTime> {
        self.questions
            .get(self.index)
            .map(|question| question.clock)
    }
    pub fn prompt(&self) -> Option<String> {
        match self.question_kind() {
            DateQuestionKind::ChooseWeekday => {
                self.date().map(|date| date.weekday_english().to_owned())
            }
            DateQuestionKind::BuildTime | DateQuestionKind::ChooseTime => {
                self.clock().map(ChineseTime::western)
            }
            _ => self.date().map(ChineseDate::western),
        }
    }
    pub fn correct_answer(&self) -> Option<String> {
        match self.question_kind() {
            DateQuestionKind::BuildHanzi | DateQuestionKind::ChooseHanzi => {
                self.date().map(ChineseDate::hanzi)
            }
            DateQuestionKind::ChooseWeekday => {
                self.date().map(|date| date.weekday_hanzi().to_owned())
            }
            DateQuestionKind::BuildTime | DateQuestionKind::ChooseTime => {
                self.clock().map(ChineseTime::hanzi)
            }
        }
    }
    pub fn tiles(&self) -> Option<Vec<String>> {
        match self.question_kind() {
            DateQuestionKind::BuildHanzi => self.date().map(ChineseDate::tiles),
            DateQuestionKind::BuildTime => self.clock().map(ChineseTime::tiles),
            _ => None,
        }
    }
    pub fn selected_tiles(&self) -> &[usize] {
        &self.selected_tiles
    }
    pub fn tile_order(&self) -> Vec<usize> {
        self.questions
            .get(self.index)
            .map_or_else(Vec::new, |question| question.tile_order.clone())
    }
    pub fn select_tile(&mut self, tile: usize) -> bool {
        if self.finished()
            || self.result.is_some()
            || !self.question_kind().is_build()
            || tile >= self.tiles().map_or(0, |tiles| tiles.len())
            || self.selected_tiles.contains(&tile)
        {
            return false;
        }
        self.selected_tiles.push(tile);
        true
    }
    pub fn insert_tile(&mut self, tile: usize, position: usize) -> bool {
        if self.finished()
            || self.result.is_some()
            || !self.question_kind().is_build()
            || tile >= self.tiles().map_or(0, |tiles| tiles.len())
            || self.selected_tiles.contains(&tile)
            || position > self.selected_tiles.len()
        {
            return false;
        }
        self.selected_tiles.insert(position, tile);
        true
    }

    pub fn move_tile(&mut self, from: usize, to: usize) -> bool {
        if self.finished()
            || self.result.is_some()
            || !self.question_kind().is_build()
            || from >= self.selected_tiles.len()
            || to >= self.selected_tiles.len()
            || from == to
        {
            return false;
        }
        let tile = self.selected_tiles.remove(from);
        self.selected_tiles.insert(to, tile);
        true
    }

    pub fn remove_tile(&mut self, position: usize) -> bool {
        if self.finished()
            || self.result.is_some()
            || !self.question_kind().is_build()
            || position >= self.selected_tiles.len()
        {
            return false;
        }
        self.selected_tiles.remove(position);
        true
    }

    pub fn reset_tiles(&mut self) -> bool {
        if self.finished()
            || self.result.is_some()
            || !self.question_kind().is_build()
            || self.selected_tiles.is_empty()
        {
            return false;
        }
        self.selected_tiles.clear();
        true
    }

    pub fn remove_last_tile(&mut self) -> bool {
        if self.finished() || self.result.is_some() || !self.question_kind().is_build() {
            return false;
        }
        self.selected_tiles.pop().is_some()
    }
    pub fn submit_tiles(&mut self) -> bool {
        if self.finished() || self.result.is_some() || !self.question_kind().is_build() {
            return false;
        }
        let Some(count) = self.tiles().map(|tiles| tiles.len()) else {
            return false;
        };
        if self.selected_tiles.len() != count {
            return false;
        }
        self.finish_answer(self.selected_tiles.iter().copied().eq(0..count));
        true
    }
    pub fn options(&self) -> Option<[String; 3]> {
        if self.finished() || self.question_kind().is_build() {
            return None;
        }
        let choices = match self.question_kind() {
            DateQuestionKind::ChooseWeekday => {
                let date = self.date()?;
                [
                    date.weekday_hanzi().to_owned(),
                    date.adjacent(false).weekday_hanzi().to_owned(),
                    date.adjacent(true).weekday_hanzi().to_owned(),
                ]
            }
            DateQuestionKind::ChooseHanzi => {
                let date = self.date()?;
                [
                    date.hanzi(),
                    date.adjacent(false).hanzi(),
                    date.adjacent(true).hanzi(),
                ]
            }
            DateQuestionKind::ChooseTime => {
                let clock = self.clock()?;

                let a = (clock.minute + 10) % 60;
                let b = (clock.minute + 20) % 60;
                [
                    clock.hanzi(),
                    clock.with_minute(a).hanzi(),
                    clock.with_minute(b).hanzi(),
                ]
            }
            _ => return None,
        };
        let correct = self.correct_option_index();
        Some([
            choices[(3 - correct) % 3].clone(),
            choices[(4 - correct) % 3].clone(),
            choices[(5 - correct) % 3].clone(),
        ])
    }
    pub fn correct_option_index(&self) -> usize {
        self.questions
            .get(self.index)
            .map_or(0, |question| question.correct_option)
    }
    pub fn choose_option(&mut self, choice: usize) -> bool {
        if self.finished()
            || self.result.is_some()
            || self.question_kind().is_build()
            || choice >= 3
        {
            return false;
        }
        self.chosen_option = Some(choice);
        self.finish_answer(choice == self.correct_option_index());
        true
    }
    fn finish_answer(&mut self, correct: bool) {
        if correct {
            self.score += 1;
        }
        self.result = Some(correct);
    }
    pub fn next(&mut self) -> bool {
        if self.result.is_none() {
            return false;
        }
        self.index += 1;
        self.selected_tiles.clear();
        self.chosen_option = None;
        self.result = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer_current(game: &mut TimeWeaver) {
        if game.question_kind().is_build() {
            let count = game.tiles().unwrap().len();
            for tile in 0..count {
                assert!(game.select_tile(tile));
            }
            assert!(game.submit_tiles());
        } else {
            assert!(game.choose_option(game.correct_option_index()));
        }
        assert_eq!(game.reviewed(), Some(true));
        assert!(game.next());
    }

    #[test]
    fn numeric_dates_render_in_natural_chinese_order() {
        assert_eq!(
            ChineseDate::new(2015, 4, 21).unwrap().hanzi(),
            "二零一五年四月二十一日"
        );
        assert_eq!(
            ChineseDate::new(2024, 2, 29).unwrap().hanzi(),
            "二零二四年二月二十九日"
        );
        assert_eq!(
            ChineseDate::new(2000, 10, 10).unwrap().hanzi(),
            "二零零零年十月十日"
        );
        assert_eq!(
            ChineseDate::new(1999, 12, 31).unwrap().hanzi(),
            "一九九九年十二月三十一日"
        );
    }

    #[test]
    fn rejects_impossible_dates_and_leap_year_errors() {
        assert!(ChineseDate::new(2023, 2, 29).is_none());
        assert!(ChineseDate::new(1900, 2, 29).is_none());
        assert!(ChineseDate::new(2000, 2, 29).is_some());
        assert!(ChineseDate::new(2025, 4, 31).is_none());
        assert!(ChineseDate::new(2025, 0, 1).is_none());
    }

    #[test]
    fn weekday_matches_known_dates_and_leap_years() {
        assert_eq!(
            ChineseDate::new(2015, 4, 21).unwrap().weekday_hanzi(),
            "星期二"
        );
        assert_eq!(
            ChineseDate::new(2024, 2, 29).unwrap().weekday_hanzi(),
            "星期四"
        );
        assert_eq!(
            ChineseDate::new(2000, 1, 1).unwrap().weekday_hanzi(),
            "星期六"
        );
        assert_eq!(
            ChineseDate::new(2026, 11, 12).unwrap().weekday_english(),
            "Thursday"
        );
    }

    #[test]
    fn adjacent_days_cross_month_year_and_leap_boundaries() {
        let date = ChineseDate::new(2024, 3, 1).unwrap();
        assert_eq!(date.adjacent(false), ChineseDate::new(2024, 2, 29).unwrap());
        let date = ChineseDate::new(1999, 12, 31).unwrap();
        assert_eq!(date.adjacent(true), ChineseDate::new(2000, 1, 1).unwrap());
    }

    #[test]
    fn clock_time_validates_bounds_and_renders_conventional_mandarin() {
        assert!(ChineseTime::new(24, 0).is_none());
        assert!(ChineseTime::new(10, 60).is_none());
        assert_eq!(ChineseTime::new(15, 20).unwrap().hanzi(), "下午三点二十分");
        assert_eq!(ChineseTime::new(9, 15).unwrap().hanzi(), "上午九点一刻");
        assert_eq!(ChineseTime::new(20, 30).unwrap().hanzi(), "晚上八点半");
        assert_eq!(ChineseTime::new(14, 2).unwrap().hanzi(), "下午两点零二分");
        assert_eq!(ChineseTime::new(0, 45).unwrap().hanzi(), "凌晨十二点三刻");
        assert_eq!(ChineseTime::new(7, 0).unwrap().hanzi(), "早上七点");
    }

    #[test]
    fn selected_lengths_preserve_balanced_question_kinds() {
        for count in [10, 15, 20] {
            let game = TimeWeaver::with_seed_and_count(42, count);
            assert_eq!(game.total(), count);
            assert_eq!(game, TimeWeaver::with_seed_and_count(42, count));
            for kind in [
                DateQuestionKind::BuildHanzi,
                DateQuestionKind::ChooseHanzi,
                DateQuestionKind::ChooseWeekday,
                DateQuestionKind::BuildTime,
                DateQuestionKind::ChooseTime,
            ] {
                assert_eq!(game.questions.iter().filter(|q| q.kind == kind).count(), count / 5);
            }
        }
    }

    #[test]
    fn seeded_sessions_are_reproducible_and_different_seeds_vary() {
        let first = TimeWeaver::with_seed(42);
        let same = TimeWeaver::with_seed(42);
        let different = TimeWeaver::with_seed(43);
        assert_eq!(first, same);
        assert_ne!(first.questions, different.questions);
        assert_eq!(first.total(), 15);
    }

    #[test]
    fn generated_sessions_have_valid_unique_dates_times_and_balanced_question_types() {
        for seed in 0..128 {
            let game = TimeWeaver::with_seed(seed);
            let mut kinds = [0usize; 5];
            let mut dates = std::collections::BTreeSet::new();
            let mut times = std::collections::BTreeSet::new();
            for question in &game.questions {
                assert!(
                    ChineseDate::new(question.date.year, question.date.month, question.date.day)
                        .is_some()
                );
                assert!(ChineseTime::new(question.clock.hour, question.clock.minute).is_some());
                assert!(dates.insert((question.date.year, question.date.month, question.date.day)));
                assert!(times.insert((question.clock.hour, question.clock.minute)));
                let category = match question.kind {
                    DateQuestionKind::BuildHanzi => 0,
                    DateQuestionKind::ChooseHanzi => 1,
                    DateQuestionKind::ChooseWeekday => 2,
                    DateQuestionKind::BuildTime => 3,
                    DateQuestionKind::ChooseTime => 4,
                };
                kinds[category] += 1;
                let mut order = question.tile_order.clone();
                order.sort_unstable();
                assert_eq!(order, (0..order.len()).collect::<Vec<_>>());
            }
            assert_eq!(kinds, [3; 5]);
        }
    }

    #[test]
    fn every_generated_choice_has_one_correct_answer_and_can_finish() {
        for seed in 0..64 {
            let mut game = TimeWeaver::with_seed(seed);
            while !game.finished() {
                let expected = game.correct_answer().unwrap();
                if let Some(choices) = game.options() {
                    assert_ne!(choices[0], choices[1]);
                    assert_ne!(choices[1], choices[2]);
                    assert_ne!(choices[0], choices[2]);
                    assert_eq!(
                        choices.iter().filter(|answer| *answer == &expected).count(),
                        1
                    );
                    assert_eq!(choices[game.correct_option_index()], expected);
                } else {
                    assert_eq!(game.tiles().unwrap().join(""), expected);
                }
                answer_current(&mut game);
            }
            assert_eq!(game.score(), 15);
            assert!(!game.next());
        }
    }

    #[test]
    fn tile_selection_reorder_reset_and_failed_answer_work() {
        let mut game = (0..100)
            .map(TimeWeaver::with_seed)
            .find(|game| game.question_kind().is_build())
            .unwrap();
        assert!(!game.submit_tiles());
        assert!(game.select_tile(0));
        assert!(game.select_tile(2));
        assert!(game.insert_tile(1, 1));
        assert_eq!(game.selected_tiles(), &[0, 1, 2]);
        assert!(!game.insert_tile(1, 0));
        assert!(game.move_tile(2, 0));
        assert_eq!(game.selected_tiles(), &[2, 0, 1]);
        assert!(game.remove_tile(1));
        assert!(game.reset_tiles());
        assert!(!game.reset_tiles());
        for tile in (0..game.tiles().unwrap().len()).rev() {
            assert!(game.select_tile(tile));
        }
        assert!(game.submit_tiles());
        assert_eq!(game.reviewed(), Some(false));
        assert_eq!(game.score(), 0);
        assert!(!game.reset_tiles());
        assert!(!game.remove_tile(0));
        assert!(game.next());
    }

    #[test]
    fn weekday_prompt_uses_english_weekday_for_random_date() {
        for seed in 0..48 {
            let mut game = TimeWeaver::with_seed(seed);
            while !game.finished() {
                if game.question_kind() == DateQuestionKind::ChooseWeekday {
                    let date = game.date().unwrap();
                    assert_eq!(game.prompt().as_deref(), Some(date.weekday_english()));
                    assert_eq!(game.correct_answer().as_deref(), Some(date.weekday_hanzi()));
                }
                answer_current(&mut game);
            }
        }
    }
}
