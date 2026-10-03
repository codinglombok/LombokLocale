//! CLDR plural rules (SPEC section 4), evaluated from the generated tables.

use crate::decimal::{self, Dec};
use crate::error::{Error, ErrorCode, Result};
use crate::tables::{Rel, RuleTable};
use crate::tag::parse_tag;

/// CLDR plural category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralCategory {
    /// `"zero"`, `"one"`, `"two"`, `"few"`, `"many"` or `"other"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::One => "one",
            Self::Two => "two",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }

    fn from_index(i: u8) -> Self {
        match i {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Few,
            _ => Self::Many,
        }
    }
}

struct Operands<'a> {
    int: &'a str,
    frac: &'a str,
}

impl Operands<'_> {
    /// (is an integer, value after modulus or None if above 10^18).
    fn value(&self, op: u8, modulus: u32) -> (bool, Option<u64>) {
        let m = u64::from(modulus);
        let digits = |s: &str| -> Option<u64> {
            if m > 0 {
                Some(decimal::digits_mod(s, m))
            } else {
                decimal::digits_value(s)
            }
        };
        let trimmed = self.frac.trim_end_matches('0');
        match op {
            b'n' => (self.frac.bytes().all(|b| b == b'0'), digits(self.int)),
            b'i' => (true, digits(self.int)),
            b'v' => (true, digits_small(self.frac.len(), m)),
            b'w' => (true, digits_small(trimmed.len(), m)),
            b'f' => (
                true,
                digits(if self.frac.is_empty() { "0" } else { self.frac }),
            ),
            b't' => (true, digits(if trimmed.is_empty() { "0" } else { trimmed })),
            _ => (true, Some(0)),
        }
    }
}

fn digits_small(v: usize, m: u64) -> Option<u64> {
    let v = v as u64;
    Some(if m > 0 { v % m } else { v })
}

fn matches(groups: &[&[Rel]], ops: &Operands<'_>) -> bool {
    groups.iter().any(|rels| {
        rels.iter().all(|r| {
            let (is_int, val) = ops.value(r.op, r.modulus);
            let inside = is_int
                && val.is_some_and(|v| {
                    r.ranges
                        .iter()
                        .any(|&(lo, hi)| u64::from(lo) <= v && v <= u64::from(hi))
                });
            inside != r.negated
        })
    })
}

fn select(table: RuleTable, locale: &str, value: &str) -> Result<PluralCategory> {
    let (int, frac) = match decimal::parse(value)? {
        Dec::Finite { int, frac, .. } => (int, frac),
        _ => return Err(Error::new(ErrorCode::BadNumber, value)),
    };
    let ops = Operands {
        int: &int,
        frac: &frac,
    };
    let entry = parse_tag(locale)
        .ok()
        .and_then(|t| {
            t.lookup_chain()
                .into_iter()
                .find_map(|c| table.iter().find(|(n, _)| *n == c))
        })
        .or_else(|| table.iter().find(|(n, _)| *n == "root"));
    if let Some((_, cats)) = entry {
        for (cat, groups) in cats.iter() {
            if matches(groups, &ops) {
                return Ok(PluralCategory::from_index(*cat));
            }
        }
    }
    Ok(PluralCategory::Other)
}

/// Cardinal plural category of `value` (a decimal string such as `"1"` or
/// `"1.50"`; trailing zeros count as visible fraction digits) in `locale`.
///
/// # Errors
/// `BAD_NUMBER` for text that is not a finite decimal.
pub fn plural_category(locale: &str, value: &str) -> Result<PluralCategory> {
    select(crate::data::PLURALS, locale, value)
}

/// Ordinal plural category (1st, 2nd, 3rd, ...) of `value` in `locale`.
///
/// # Errors
/// `BAD_NUMBER` for text that is not a finite decimal.
pub fn ordinal_category(locale: &str, value: &str) -> Result<PluralCategory> {
    select(crate::data::ORDINALS, locale, value)
}
