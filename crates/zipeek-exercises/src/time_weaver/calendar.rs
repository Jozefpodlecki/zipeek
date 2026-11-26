#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChineseDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
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

    pub(super) fn adjacent(self, forward: bool) -> Self {
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

pub(super) fn days_in_month(year: u16, month: u8) -> u8 {
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
    pub hour: u8,
    pub minute: u8,
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

    pub(super) fn with_minute(self, minute: u8) -> Self {
        Self { minute, ..self }
    }
}
