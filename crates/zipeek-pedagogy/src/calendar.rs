#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarError {
    InvalidDate,
}

pub fn local_day_number(iso_date: &str) -> Result<i64, CalendarError> {
    let parts: Vec<_> = iso_date.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return Err(CalendarError::InvalidDate);
    }
    let parse = |value: &str| value.parse::<i64>().map_err(|_| CalendarError::InvalidDate);
    let mut year = parse(parts[0])?;
    let month = parse(parts[1])?;
    let day = parse(parts[2])?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=12).contains(&month) || day < 1 || day > month_days[(month - 1) as usize] {
        return Err(CalendarError::InvalidDate);
    }
    year -= i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let year_of_era_day = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Ok(era * 146097 + year_of_era_day - 719468)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_calendar_inputs_are_rejected_without_normalization() {
        for invalid in [
            "2025-02-29",
            "1900-02-29",
            "2026-04-31",
            "2026-13-01",
            "2026-00-01",
            "2026-01-00",
            "2026-1-01",
            "2026-01-1",
            "2026-01-01Z",
            "2026/01/01",
        ] {
            assert_eq!(
                local_day_number(invalid),
                Err(CalendarError::InvalidDate),
                "{invalid}"
            );
        }
        assert!(local_day_number("2000-02-29").is_ok());
    }

    #[test]
    fn leap_and_boundary_dates() {
        assert_eq!(local_day_number("1970-01-01"), Ok(0));
        assert_eq!(
            local_day_number("2024-03-01").unwrap() - local_day_number("2024-02-29").unwrap(),
            1
        );
        assert_eq!(
            local_day_number("2026-01-01").unwrap() - local_day_number("2025-12-31").unwrap(),
            1
        );
        assert_eq!(
            local_day_number("2026-02-29"),
            Err(CalendarError::InvalidDate)
        );
        assert_eq!(
            local_day_number("2026-2-01"),
            Err(CalendarError::InvalidDate)
        );
    }

    #[test]
    fn leap_century_and_utc_epoch_boundaries() {
        assert_eq!(
            local_day_number("2000-02-29").unwrap() + 1,
            local_day_number("2000-03-01").unwrap()
        );
        assert_eq!(
            local_day_number("1900-02-29"),
            Err(CalendarError::InvalidDate)
        );
        assert_eq!(local_day_number("1970-01-01"), Ok(0));
        assert_eq!(local_day_number("1969-12-31"), Ok(-1));
        assert_eq!(
            local_day_number("2026-11-08").unwrap() + 1,
            local_day_number("2026-11-09").unwrap()
        );
    }

    #[test]
    fn malformed_or_non_ascii_days_cannot_be_used_for_mastery_spacing() {
        for invalid in [
            "2026-11-０8",
            "2026-11-08 ",
            " 2026-11-08",
            "2026-11-08\n",
            "2026-11-08T00:00:00",
        ] {
            assert_eq!(local_day_number(invalid), Err(CalendarError::InvalidDate));
        }
        assert_eq!(
            local_day_number("2026-11-09").unwrap() - local_day_number("2026-11-08").unwrap(),
            1
        );
    }
}
