//! Number, percent and currency formatting (SPEC section 5).

use crate::decimal::{self, Dec};
use crate::error::{Error, ErrorCode, Result};
use crate::tables::LocaleData;
use crate::tag::locale_data;
use alloc::string::{String, ToString};

/// Formatting style.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Style {
    /// Plain decimal number (0 to 3 fraction digits by default).
    #[default]
    Decimal,
    /// Percent; the value is multiplied by 100 (0 fraction digits by default).
    Percent,
    /// Currency with the ISO 4217 code (its CLDR fraction digits by default).
    Currency(String),
}

/// Options of [`format_number`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NumberOptions {
    pub style: Style,
    /// Minimum fraction digits, 0..=20.
    pub min_fraction: Option<u8>,
    /// Maximum fraction digits, 0..=20.
    pub max_fraction: Option<u8>,
}

impl NumberOptions {
    /// Decimal style with default digits.
    pub fn decimal() -> Self {
        Self::default()
    }

    /// Percent style with default digits.
    pub fn percent() -> Self {
        Self {
            style: Style::Percent,
            ..Self::default()
        }
    }

    /// Currency style with default digits.
    pub fn currency(code: &str) -> Self {
        Self {
            style: Style::Currency(code.to_string()),
            ..Self::default()
        }
    }

    /// Sets the fraction digit range.
    pub fn fraction(mut self, min: Option<u8>, max: Option<u8>) -> Self {
        self.min_fraction = min;
        self.max_fraction = max;
        self
    }
}

/// Splits one subpattern into (prefix, number part, suffix).
fn split_pattern(p: &str) -> (&str, &str, &str) {
    let mut quoted = false;
    let (mut start, mut end) = (None, 0);
    for (i, c) in p.char_indices() {
        if c == '\'' {
            quoted = !quoted;
        } else if !quoted && matches!(c, '#' | '0' | ',' | '.') {
            if start.is_none() {
                start = Some(i);
            }
            end = i + 1;
        }
    }
    let start = start.unwrap_or(p.len());
    (&p[..start], &p[start..end.max(start)], &p[end.max(start)..])
}

fn grouping(numpart: &str) -> (usize, usize) {
    let ip = numpart.split('.').next().unwrap_or("");
    let groups: alloc::vec::Vec<&str> = ip.split(',').collect();
    let primary = if groups.len() > 1 {
        groups[groups.len() - 1].len()
    } else {
        0
    };
    let secondary = if groups.len() > 2 {
        groups[groups.len() - 2].len()
    } else {
        primary
    };
    (primary, secondary)
}

fn group_int(ip: &str, primary: usize, secondary: usize, min_grouping: usize, sep: &str) -> String {
    if primary == 0 || ip.len() < primary + min_grouping {
        return ip.to_string();
    }
    let (mut head, tail) = ip.split_at(ip.len() - primary);
    let mut parts = alloc::vec![tail];
    while head.len() > secondary {
        let (h, t) = head.split_at(head.len() - secondary);
        parts.insert(0, t);
        head = h;
    }
    if !head.is_empty() {
        parts.insert(0, head);
    }
    parts.join(sep)
}

fn expand_affix(raw: &str, d: &LocaleData, symbol: &str) -> String {
    let mut out = String::new();
    let mut quoted = false;
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\'' {
            if chars.peek() == Some(&'\'') {
                chars.next();
                out.push('\'');
                continue;
            }
            quoted = !quoted;
        } else if quoted {
            out.push(c);
        } else {
            match c {
                '¤' => out.push_str(symbol),
                '%' => out.push_str(d.percent),
                '-' => out.push_str(d.minus),
                _ => out.push(c),
            }
        }
    }
    out
}

fn currency_digits(code: &str) -> usize {
    crate::data::CURRENCY_DIGITS
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(2, |(_, d)| usize::from(*d))
}

/// Formats `value` (a decimal string, `NaN`, `Infinity` or `-Infinity`) for
/// `locale`. Rounding is half away from zero; grouping follows the locale
/// pattern and minimum grouping digits.
///
/// # Errors
/// `BAD_NUMBER` for unreadable values, `BAD_OPTION` for a missing or
/// malformed currency code or an invalid fraction digit range.
pub fn format_number(locale: &str, value: &str, opts: &NumberOptions) -> Result<String> {
    let d = locale_data(locale);
    let mut code = String::new();
    let (dmin, dmax, pattern) = match &opts.style {
        Style::Decimal => (0, 3, d.decimal_pattern),
        Style::Percent => (0, 0, d.percent_pattern),
        Style::Currency(c) => {
            if c.len() != 3 || !c.bytes().all(|b| b.is_ascii_alphabetic()) {
                return Err(Error::new(ErrorCode::BadOption, "currency"));
            }
            code = c.to_ascii_uppercase();
            let digits = currency_digits(&code);
            (digits, digits, d.currency_pattern)
        }
    };
    let (minf, maxf) = match (
        opts.min_fraction.map(usize::from),
        opts.max_fraction.map(usize::from),
    ) {
        (None, None) => (dmin, dmax),
        (Some(a), None) => (a, a.max(dmax)),
        (None, Some(b)) => (dmin.min(b), b),
        (Some(a), Some(b)) => (a, b),
    };
    if minf > 20 || maxf > 20 || minf > maxf {
        return Err(Error::new(ErrorCode::BadOption, "fractionDigits"));
    }
    let (pos, negp) = match pattern.split_once(';') {
        Some((p, n)) => (p, Some(n)),
        None => (pattern, None),
    };
    let (prefix, numpart, suffix) = split_pattern(pos);
    let (neg, body) = match decimal::parse(value)? {
        Dec::NaN => (false, d.nan.to_string()),
        Dec::Inf { neg } => (neg, d.infinity.to_string()),
        Dec::Finite {
            neg,
            mut int,
            mut frac,
        } => {
            if opts.style == Style::Percent {
                frac.push_str("00");
                let shifted = alloc::format!("{}{}", int, &frac[..2]);
                let t = shifted.trim_start_matches('0');
                int = if t.is_empty() {
                    "0".to_string()
                } else {
                    t.to_string()
                };
                frac = frac[2..].to_string();
            }
            let (ip, mut fp) = decimal::round(&int, &frac, maxf);
            while fp.len() < minf {
                fp.push('0');
            }
            while fp.len() > minf && fp.ends_with('0') {
                fp.pop();
            }
            let (primary, secondary) = grouping(numpart);
            let mut body = group_int(&ip, primary, secondary, d.min_grouping, d.group);
            if !fp.is_empty() {
                body.push_str(d.decimal);
                body.push_str(&fp);
            }
            (neg, body)
        }
    };
    let (np, ns) = match (neg, negp) {
        (true, Some(n)) => {
            let (a, _, b) = split_pattern(n);
            (a.to_string(), b.to_string())
        }
        (true, None) => (alloc::format!("-{}", prefix), suffix.to_string()),
        (false, _) => (prefix.to_string(), suffix.to_string()),
    };
    if let Style::Currency(_) = opts.style {
        let entry = d.currency_symbols.iter().find(|(c, ..)| *c == code);
        let (sym, first_sz, last_sz) =
            entry.map_or((code.as_str(), false, false), |e| (e.1, e.2, e.3));
        let mut pre = expand_affix(&np, d, sym);
        let mut suf = expand_affix(&ns, d, sym);
        let starts_digit = body.chars().next().is_some_and(|c| c.is_ascii_digit());
        let ends_digit = body.chars().last().is_some_and(|c| c.is_ascii_digit());
        if np.ends_with('¤') && !last_sz && starts_digit {
            pre.push('\u{a0}');
        }
        if ns.starts_with('¤') && !first_sz && ends_digit {
            suf.insert(0, '\u{a0}');
        }
        return Ok(alloc::format!("{}{}{}", pre, body, suf));
    }
    Ok(alloc::format!(
        "{}{}{}",
        expand_affix(&np, d, ""),
        body,
        expand_affix(&ns, d, "")
    ))
}
