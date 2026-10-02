//! Date formatting with CLDR patterns (SPEC section 6).

use crate::error::{Error, ErrorCode, Result};
use crate::tag::locale_data;
use alloc::string::String;

/// CLDR date style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateStyle {
    Full,
    Long,
    #[default]
    Medium,
    Short,
}

impl DateStyle {
    /// Parses `"full"`, `"long"`, `"medium"` or `"short"`.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "full" => Some(Self::Full),
            "long" => Some(Self::Long),
            "medium" => Some(Self::Medium),
            "short" => Some(Self::Short),
            _ => None,
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// A proleptic Gregorian date between 0001-01-01 and 9999-12-31.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

fn is_leap(y: u32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

fn days_in_month(y: u32, m: u32) -> u32 {
    match m {
        2 if is_leap(y) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Date {
    /// Parses `YYYY-MM-DD` (exactly ten characters).
    ///
    /// # Errors
    /// `BAD_DATE` for any other text or a day that does not exist.
    pub fn parse(s: &str) -> Result<Self> {
        let b = s.as_bytes();
        let bad = || Error::new(ErrorCode::BadDate, s);
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return Err(bad());
        }
        let num = |r: core::ops::Range<usize>| -> Option<u32> {
            b[r].iter().try_fold(0u32, |a, &c| {
                c.is_ascii_digit().then(|| a * 10 + u32::from(c - b'0'))
            })
        };
        let (y, m, d) = (
            num(0..4).ok_or_else(bad)?,
            num(5..7).ok_or_else(bad)?,
            num(8..10).ok_or_else(bad)?,
        );
        if y == 0 || !(1..=12).contains(&m) || d == 0 || d > days_in_month(y, m) {
            return Err(bad());
        }
        Ok(Self {
            year: y as u16,
            month: m as u8,
            day: d as u8,
        })
    }

    /// Day of the week, 0 = Sunday.
    pub fn weekday(self) -> usize {
        let y = u64::from(self.year) - 1;
        let mut days = y * 365 + y / 4 - y / 100 + y / 400;
        for m in 1..u32::from(self.month) {
            days += u64::from(days_in_month(u32::from(self.year), m));
        }
        days += u64::from(self.day) - 1;
        ((days + 1) % 7) as usize
    }
}

fn pad(out: &mut String, v: u32, width: usize) {
    let s = alloc::format!("{}", v);
    for _ in s.len()..width {
        out.push('0');
    }
    out.push_str(&s);
}

/// Formats `date` (`YYYY-MM-DD`) for `locale` in `style`.
///
/// # Errors
/// `BAD_DATE` for an invalid date.
pub fn format_date(locale: &str, date: &str, style: DateStyle) -> Result<String> {
    let dt = Date::parse(date)?;
    let d = locale_data(locale);
    let pat: alloc::vec::Vec<char> = d.date_formats[style.index()].chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < pat.len() {
        let c = pat[i];
        if c == '\'' {
            let mut j = i + 1;
            while j < pat.len() {
                if pat[j] == '\'' {
                    if j + 1 < pat.len() && pat[j + 1] == '\'' {
                        out.push('\'');
                        j += 2;
                        continue;
                    }
                    break;
                }
                out.push(pat[j]);
                j += 1;
            }
            if j == i + 1 {
                out.push('\'');
            }
            i = j + 1;
            continue;
        }
        if c.is_ascii_alphabetic() {
            let mut j = i;
            while j < pat.len() && pat[j] == c {
                j += 1;
            }
            let n = j - i;
            match c {
                'G' => out.push_str(d.era),
                'y' if n == 2 => pad(&mut out, u32::from(dt.year) % 100, 2),
                'y' => pad(&mut out, u32::from(dt.year), n),
                'M' | 'L' if n <= 2 => pad(&mut out, u32::from(dt.month), n),
                'M' | 'L' => out.push_str(d.months[usize::from(n != 3)][usize::from(dt.month) - 1]),
                'd' => pad(&mut out, u32::from(dt.day), n),
                'E' => out.push_str(d.days[usize::from(n == 4)][dt.weekday()]),
                _ => unreachable!("pattern letter {c} is not used by the CLDR 47 subset"),
            }
            i = j;
            continue;
        }
        out.push(c);
        i += 1;
    }
    Ok(out)
}
