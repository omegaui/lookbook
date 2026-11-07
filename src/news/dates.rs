//! Calendar days from feed dates. Feeds use ISO 8601 ("2026-11-03T09:47:53Z"),
//! RFC 2822 ("Tue, 03 Nov 2026 00:00:00 +0000") or a plain "2026-11-03".
//! Only the day matters here, so times and time zones are dropped.

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Day {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Day {
    /// Reads "2026-11-03", with or without a time after it.
    pub fn from_iso(text: &str) -> Option<Self> {
        let date = text.get(..10)?;
        let mut parts = date.split('-');
        let year = parts.next()?.parse().ok()?;
        let month = parts.next()?.parse().ok()?;
        let day = parts.next()?.parse().ok()?;
        Self::checked(year, month, day)
    }

    /// Reads "Tue, 03 Nov 2026 00:00:00 +0000".
    pub fn from_rfc2822(text: &str) -> Option<Self> {
        let after_weekday = text.split_once(',').map_or(text, |(_, rest)| rest);
        let mut parts = after_weekday.split_whitespace();
        let day = parts.next()?.parse().ok()?;
        let month_name = parts.next()?;
        let month = MONTHS.iter().position(|name| name.eq_ignore_ascii_case(month_name))? as u32 + 1;
        let year = parts.next()?.parse().ok()?;
        Self::checked(year, month, day)
    }

    fn checked(year: i32, month: u32, day: u32) -> Option<Self> {
        let valid = (1..=12).contains(&month) && (1..=31).contains(&day);
        valid.then_some(Self { year, month, day })
    }

    /// Days between `self` and a later `other`.
    pub fn days_until(self, other: Self) -> i64 {
        other.days_since_epoch() - self.days_since_epoch()
    }

    /// Days since 1970-01-01 in the proleptic Gregorian calendar
    /// (Howard Hinnant's `days_from_civil`).
    fn days_since_epoch(self) -> i64 {
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let month = i64::from(self.month);
        let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_iso_dates_with_and_without_time() {
        let day = Day { year: 2026, month: 11, day: 3 };
        assert_eq!(Day::from_iso("2026-11-03"), Some(day));
        assert_eq!(Day::from_iso("2026-11-03T09:47:53Z"), Some(day));
    }

    #[test]
    fn reads_rfc2822_dates() {
        let day = Day::from_rfc2822("Sat, 10 Oct 2026 10:37:59 +0200");
        assert_eq!(day, Some(Day { year: 2026, month: 10, day: 10 }));
    }

    #[test]
    fn counts_days_across_months_and_years() {
        let start = Day { year: 2026, month: 10, day: 7 };
        assert_eq!(start.days_until(Day { year: 2026, month: 11, day: 7 }), 31);
        let new_year = Day { year: 2025, month: 12, day: 31 };
        assert_eq!(new_year.days_until(Day { year: 2026, month: 1, day: 1 }), 1);
        let leap = Day { year: 2028, month: 2, day: 28 };
        assert_eq!(leap.days_until(Day { year: 2028, month: 3, day: 1 }), 2);
    }

    #[test]
    fn rejects_nonsense() {
        assert_eq!(Day::from_iso("not a date"), None);
        assert_eq!(Day::from_rfc2822("Tue, 03 Foo 2026"), None);
    }
}
