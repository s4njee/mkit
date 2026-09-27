//! A timezone-free Gregorian calendar date.

/// A Gregorian calendar date with years in `1..=9999`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CivilDate {
    year: u16,
    month: u8,
    day: u8,
}

impl CivilDate {
    pub const MIN: Self = Self { year: 1, month: 1, day: 1 };
    pub const MAX: Self = Self { year: 9999, month: 12, day: 31 };

    pub const fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        if year < 1
            || year > 9999
            || month < 1
            || month > 12
            || day < 1
            || day > days_in_month(year, month)
        {
            None
        } else {
            Some(Self { year, month, day })
        }
    }

    pub const fn year(self) -> u16 {
        self.year
    }
    pub const fn month(self) -> u8 {
        self.month
    }
    pub const fn day(self) -> u8 {
        self.day
    }

    /// Monday is zero; Sunday is six.
    pub fn weekday_monday_zero(self) -> u8 {
        (self.ordinal() % 7) as u8
    }

    pub fn add_days(self, days: i32) -> Option<Self> {
        let ordinal = self.ordinal() as i64 + days as i64;
        let max = days_before_year(10000) as i64;
        if ordinal < 0 || ordinal >= max {
            return None;
        }
        Some(from_ordinal(ordinal as u32))
    }

    pub fn add_months(self, months: i32) -> Option<Self> {
        let index = (self.year as i32 - 1) * 12 + self.month as i32 - 1 + months;
        if !(0..(9999 * 12)).contains(&index) {
            return None;
        }
        let year = (index / 12 + 1) as u16;
        let month = (index % 12 + 1) as u8;
        Self::new(year, month, self.day.min(days_in_month(year, month)))
    }

    fn ordinal(self) -> u32 {
        days_before_year(self.year as u32)
            + days_before_month(self.year, self.month)
            + self.day as u32
            - 1
    }
}

const fn leap(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
const fn days_before_year(year: u32) -> u32 {
    let y = year - 1;
    y * 365 + y / 4 - y / 100 + y / 400
}
const fn days_before_month(year: u16, month: u8) -> u32 {
    let mut n = 0;
    let mut m = 1;
    while m < month {
        n += days_in_month(year, m) as u32;
        m += 1;
    }
    n
}
fn from_ordinal(mut ordinal: u32) -> CivilDate {
    let mut year = 1u16;
    while ordinal >= if leap(year) { 366 } else { 365 } {
        ordinal -= if leap(year) { 366 } else { 365 };
        year += 1;
    }
    let mut month = 1u8;
    while ordinal >= days_in_month(year, month) as u32 {
        ordinal -= days_in_month(year, month) as u32;
        month += 1;
    }
    CivilDate { year, month, day: ordinal as u8 + 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_and_arithmetic_clamps() {
        assert_eq!(CivilDate::new(1900, 2, 29), None);
        assert_eq!(CivilDate::new(2000, 2, 29).unwrap().add_days(1), CivilDate::new(2000, 3, 1));
        assert_eq!(CivilDate::new(2024, 1, 31).unwrap().add_months(1), CivilDate::new(2024, 2, 29));
    }
    #[test]
    fn weekdays_are_monday_based() {
        assert_eq!(CivilDate::new(2024, 1, 1).unwrap().weekday_monday_zero(), 0);
        assert_eq!(CivilDate::new(2024, 1, 7).unwrap().weekday_monday_zero(), 6);
    }
}
