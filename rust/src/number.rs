//! Number and currency formatting (CLDR-subset, §7 of ARCHITECTURE_UTAMA).
//!
//! Deliberately covers grouping + decimal separator conventions rather
//! than the full CLDR number-pattern grammar, matching the "CLDR subset"
//! scope called out in the architecture doc.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Grouping/decimal separator convention for a locale's base language.
struct NumberFormat {
    group_sep: char,
    decimal_sep: char,
}

fn number_format_for(locale: &str) -> NumberFormat {
    let lang = locale.split(['-', '_']).next().unwrap_or(locale);
    match lang {
        // Comma-thousands, dot-decimal.
        "en" | "zh" | "ja" | "ko" => NumberFormat {
            group_sep: ',',
            decimal_sep: '.',
        },
        // Dot-thousands, comma-decimal (most of Europe + Indonesia).
        "id" | "de" | "nl" | "it" | "es" | "pt" | "ru" | "tr" | "vi" => NumberFormat {
            group_sep: '.',
            decimal_sep: ',',
        },
        // Space-thousands, comma-decimal.
        "fr" | "sv" | "pl" | "fi" => NumberFormat {
            group_sep: '\u{00A0}', // non-breaking space
            decimal_sep: ',',
        },
        _ => NumberFormat {
            group_sep: ',',
            decimal_sep: '.',
        },
    }
}

/// Format an integer with locale-appropriate grouping.
pub fn format_integer(value: i64, locale: &str) -> String {
    let fmt = number_format_for(locale);
    let neg = value < 0;
    let digits = if value == i64::MIN {
        // Avoid overflow on abs(); i64::MIN has no positive counterpart.
        "9223372036854775808".to_string()
    } else {
        value.unsigned_abs().to_string()
    };
    let grouped = group_digits(&digits, fmt.group_sep);
    if neg {
        alloc::format!("-{}", grouped)
    } else {
        grouped
    }
}

fn group_digits(digits: &str, sep: char) -> String {
    let bytes: Vec<char> = digits.chars().collect();
    let mut out = String::new();
    let len = bytes.len();
    for (i, c) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(sep);
        }
        out.push(*c);
    }
    out
}

/// Maximum fraction digits honoured by [`format_float`] (f64 carries ~15-17
/// significant digits; more would be noise and could overflow the fixed-point
/// scale).
pub const MAX_DECIMALS: usize = 15;

/// Magnitude above which fixed-point formatting is abandoned for scientific
/// notation (keeps `|value| * 10^decimals` inside `i128`).
const FIXED_LIMIT: f64 = 1e20;

/// Format a floating point value with `decimals` fraction digits (clamped to
/// [`MAX_DECIMALS`]) and locale-appropriate grouping/decimal separators.
/// `NaN` -> `"NaN"`, infinities -> `"∞"`/`"-∞"`, magnitudes >= 1e20 use
/// scientific notation (`1e30`), and negative values that round to zero print
/// as `0` (no `-0`).
pub fn format_float(value: f64, decimals: usize, locale: &str) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-\u{221E}" } else { "\u{221E}" }.to_string();
    }
    let fmt = number_format_for(locale);
    let decimals = decimals.min(MAX_DECIMALS);
    let abs = if value < 0.0 { -value } else { value };
    if abs >= FIXED_LIMIT {
        return alloc::format!("{:e}", value);
    }

    let mut scale: i128 = 1;
    for _ in 0..decimals {
        scale *= 10;
    }
    let scaled = (abs * (scale as f64) + 0.5) as i128; // half-up on |value|
    let int_part = scaled / scale;
    let frac_part = scaled - int_part * scale;
    let neg = value < 0.0 && scaled != 0;

    let mut out = String::new();
    if neg {
        out.push('-');
    }
    out.push_str(&group_digits(&int_part.to_string(), fmt.group_sep));
    if decimals > 0 {
        out.push(fmt.decimal_sep);
        out.push_str(&alloc::format!("{:0width$}", frac_part, width = decimals));
    }
    out
}

/// Format a minor-unit-free currency amount, e.g. `format_currency(1234.5, "IDR", "id")`
/// -> `"Rp1.234,50"`. Symbol table covers a small common set; unknown codes
/// fall back to `"<CODE> <amount>"`.
pub fn format_currency(value: f64, currency_code: &str, locale: &str) -> String {
    let amount = format_float(value, 2, locale);
    let symbol = currency_symbol(currency_code);
    let lang = locale.split(['-', '_']).next().unwrap_or(locale);
    match lang {
        "en" => alloc::format!("{}{}", symbol, amount),
        "id" => alloc::format!("{}{}", symbol, amount),
        "fr" | "sv" | "pl" | "fi" => alloc::format!("{}\u{00A0}{}", amount, symbol),
        _ => alloc::format!("{} {}", symbol, amount),
    }
}

fn currency_symbol(code: &str) -> &'static str {
    match code {
        "USD" => "$",
        "IDR" => "Rp",
        "EUR" => "\u{20AC}",
        "GBP" => "\u{00A3}",
        "JPY" => "\u{00A5}",
        "CNY" => "\u{00A5}",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_grouping_en() {
        assert_eq!(format_integer(1234567, "en"), "1,234,567");
    }

    #[test]
    fn integer_grouping_id() {
        assert_eq!(format_integer(1234567, "id"), "1.234.567");
    }

    #[test]
    fn negative_integer() {
        assert_eq!(format_integer(-9999, "en"), "-9,999");
    }

    #[test]
    fn float_formatting_id() {
        assert_eq!(format_float(1234.5, 2, "id"), "1.234,50");
    }

    #[test]
    fn float_formatting_en() {
        assert_eq!(format_float(1234.5, 2, "en"), "1,234.50");
    }

    #[test]
    fn float_edge_cases_never_panic() {
        assert_eq!(format_float(f64::NAN, 2, "en"), "NaN");
        assert_eq!(format_float(f64::INFINITY, 2, "en"), "\u{221E}");
        assert_eq!(format_float(f64::NEG_INFINITY, 2, "id"), "-\u{221E}");
        assert_eq!(format_float(-0.004, 2, "en"), "0.00");
        assert_eq!(format_float(-0.005, 2, "en"), "-0.01");
        assert_eq!(format_float(1e30, 2, "en"), "1e30");
        assert_eq!(format_float(f64::MAX, 2, "en"), format!("{:e}", f64::MAX));
        // decimals is clamped instead of overflowing the fixed-point scale
        assert_eq!(format_float(1.0, 40, "en").len(), 2 + MAX_DECIMALS);
        assert_eq!(format_float(0.0, 0, "en"), "0");
        assert_eq!(format_float(999.995, 2, "en"), "1,000.00");
    }

    #[test]
    fn currency_idr() {
        assert_eq!(format_currency(15000.0, "IDR", "id"), "Rp15.000,00");
    }

    #[test]
    fn currency_usd() {
        assert_eq!(format_currency(1999.99, "USD", "en"), "$1,999.99");
    }
}
