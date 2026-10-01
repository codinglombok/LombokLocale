//! Minimal locale-aware date formatting. Takes plain `(year, month, day)`
//! triples rather than depending on a datetime crate, keeping the module
//! zero-dep. Month/weekday name tables cover the locales shipped in
//! `locales/` today; unlisted locales fall back to English names.

use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleDate {
    pub year: i32,
    pub month: u8, // 1-12
    pub day: u8,   // 1-31
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateStyle {
    /// `2026-09-27`
    Iso,
    /// `27/09/2026` (id) or `09/27/2026` (en)
    Short,
    /// `27 September 2026`
    Long,
}

fn month_names(locale: &str) -> [&'static str; 12] {
    let lang = locale.split(['-', '_']).next().unwrap_or(locale);
    match lang {
        "id" => [
            "Januari",
            "Februari",
            "Maret",
            "April",
            "Mei",
            "Juni",
            "Juli",
            "Agustus",
            "September",
            "Oktober",
            "November",
            "Desember",
        ],
        _ => [
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
        ],
    }
}

/// Format `date` for `locale` in the given `style`.
pub fn format_date(date: SimpleDate, locale: &str, style: DateStyle) -> String {
    let lang = locale.split(['-', '_']).next().unwrap_or(locale);
    match style {
        DateStyle::Iso => alloc::format!("{:04}-{:02}-{:02}", date.year, date.month, date.day),
        DateStyle::Short => {
            if lang == "en" {
                alloc::format!("{:02}/{:02}/{:04}", date.month, date.day, date.year)
            } else {
                alloc::format!("{:02}/{:02}/{:04}", date.day, date.month, date.year)
            }
        }
        DateStyle::Long => {
            let names = month_names(locale);
            let name = names
                .get((date.month as usize).saturating_sub(1))
                .copied()
                .unwrap_or("?");
            match lang {
                "id" => alloc::format!("{} {} {}", date.day, name, date.year),
                _ => alloc::format!("{} {}, {}", name, date.day, date.year),
            }
        }
    }
}

/// Days in `month` of `year` in the proleptic Gregorian calendar.
fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            if leap {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Parse a strict ISO 8601 calendar date `YYYY-MM-DD` (4-digit year, 2-digit
/// month and day, valid Gregorian calendar day, leap years honoured).
/// Returns `None` for anything else.
pub fn parse_iso_date(s: &str) -> Option<SimpleDate> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let digits = |r: core::ops::Range<usize>| -> Option<u32> {
        let part = &b[r];
        if part.iter().all(|c| c.is_ascii_digit()) {
            Some(part.iter().fold(0u32, |a, c| a * 10 + (*c - b'0') as u32))
        } else {
            None
        }
    };
    let year = digits(0..4)? as i32;
    let month = digits(5..7)? as u8;
    let day = digits(8..10)? as u8;
    if month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    Some(SimpleDate { year, month, day })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_format() {
        let d = SimpleDate {
            year: 2026,
            month: 9,
            day: 27,
        };
        assert_eq!(format_date(d, "id", DateStyle::Iso), "2026-09-27");
    }

    #[test]
    fn short_format_id_vs_en() {
        let d = SimpleDate {
            year: 2026,
            month: 9,
            day: 27,
        };
        assert_eq!(format_date(d, "id", DateStyle::Short), "27/09/2026");
        assert_eq!(format_date(d, "en", DateStyle::Short), "09/27/2026");
    }

    #[test]
    fn long_format_id() {
        let d = SimpleDate {
            year: 2026,
            month: 9,
            day: 27,
        };
        assert_eq!(format_date(d, "id", DateStyle::Long), "27 September 2026");
    }

    #[test]
    fn long_format_en() {
        let d = SimpleDate {
            year: 2026,
            month: 1,
            day: 5,
        };
        assert_eq!(format_date(d, "en", DateStyle::Long), "January 5, 2026");
    }

    #[test]
    fn parse_roundtrip() {
        let d = parse_iso_date("2026-09-27").unwrap();
        assert_eq!(
            d,
            SimpleDate {
                year: 2026,
                month: 9,
                day: 27
            }
        );
        assert_eq!(format_date(d, "id", DateStyle::Iso), "2026-09-27");
    }

    #[test]
    fn parse_rejects_malformed() {
        for bad in [
            "2026/09/27",
            "2026-13-01",
            "2026-9-7",
            "2026-09-27 ",
            "-026-09-27",
            "2026-00-10",
            "+026-09-27",
            "２０２６-09-27",
            "",
        ] {
            assert!(parse_iso_date(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn parse_validates_calendar_days() {
        assert!(parse_iso_date("2026-02-29").is_none());
        assert!(parse_iso_date("2028-02-29").is_some());
        assert!(parse_iso_date("1900-02-29").is_none());
        assert!(parse_iso_date("2000-02-29").is_some());
        assert!(parse_iso_date("2026-04-31").is_none());
        assert!(parse_iso_date("2026-12-31").is_some());
    }
}
