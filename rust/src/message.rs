//! Minimal MessageFormat 2 runtime.
//!
//! Supports the subset the ecosystem actually needs (ARCHITECTURE §7):
//! simple placeholders `{name}`, plural selection
//! `{count, plural, one{...} other{...}}`, and generic selection
//! `{gender, select, male{...} female{...} *{...}}`. Not a full MF2
//! implementation (no functions, no nested declarations) — deliberately
//! kept zero-dep and small enough to run in `no_std + alloc`.

use crate::plural::{ordinal_category, plural_category};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    Str(String),
    Int(i64),
    UInt(u64),
    Float(f64),
}

impl ArgValue {
    fn to_display(&self) -> String {
        match self {
            ArgValue::Str(s) => s.clone(),
            ArgValue::Int(i) => i.to_string(),
            ArgValue::UInt(u) => u.to_string(),
            ArgValue::Float(f) => f.to_string(),
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            ArgValue::UInt(u) => Some(*u),
            ArgValue::Int(i) if *i >= 0 => Some(*i as u64),
            ArgValue::Float(f) if *f >= 0.0 => Some(*f as u64),
            _ => None,
        }
    }
}

/// A single named argument, as a (name, value) pair — kept as a `Vec`
/// instead of a `HashMap` so the module works under `no_std + alloc`
/// without pulling in `std::collections`.
pub type Args<'a> = &'a [(&'a str, ArgValue)];

fn lookup<'a>(args: Args<'a>, name: &str) -> Option<&'a ArgValue> {
    args.iter().find(|(n, _)| *n == name).map(|(_, v)| v)
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageError {
    UnbalancedBraces,
    UnknownVariable(String),
    MalformedSelector(String),
    /// Pattern nesting exceeded [`MAX_DEPTH`].
    TooDeep,
}

/// Maximum nesting of selectors inside a pattern.
pub const MAX_DEPTH: usize = 16;

/// Format `pattern` against `args` for `locale` (used for plural-category
/// resolution).
pub fn format(locale: &str, pattern: &str, args: Args) -> Result<String, MessageError> {
    format_depth(locale, pattern, args, 0)
}

fn format_depth(
    locale: &str,
    pattern: &str,
    args: Args,
    depth: usize,
) -> Result<String, MessageError> {
    if depth > MAX_DEPTH {
        return Err(MessageError::TooDeep);
    }
    let mut out = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '{' {
            let (block, next) = read_block(&chars, i)?;
            out.push_str(&render_block(locale, &block, args, depth)?);
            i = next;
        } else if c == '}' {
            return Err(MessageError::UnbalancedBraces);
        } else {
            out.push(c);
            i += 1;
        }
    }
    Ok(out)
}

/// Read a balanced `{...}` block starting at `start` (which must be `{`).
/// Returns the inner content and the index just past the closing `}`.
fn read_block(chars: &[char], start: usize) -> Result<(String, usize), MessageError> {
    let mut depth = 0i32;
    let mut i = start;
    let mut inner = String::new();
    loop {
        if i >= chars.len() {
            return Err(MessageError::UnbalancedBraces);
        }
        let c = chars[i];
        if c == '{' {
            depth += 1;
            if depth > 1 {
                inner.push(c);
            }
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Ok((inner, i + 1));
            }
            inner.push(c);
        } else {
            inner.push(c);
        }
        i += 1;
    }
}

fn render_block(
    locale: &str,
    inner: &str,
    args: Args,
    depth: usize,
) -> Result<String, MessageError> {
    let trimmed = inner.trim();
    // Simple placeholder: {name}
    if !trimmed.contains(',') {
        let val = lookup(args, trimmed)
            .ok_or_else(|| MessageError::UnknownVariable(trimmed.to_string()))?;
        return Ok(val.to_display());
    }

    // Selector: {var, plural, ...} or {var, select, ...}
    let mut parts = trimmed.splitn(3, ',');
    let var = parts.next().unwrap_or("").trim();
    let kind = parts.next().unwrap_or("").trim();
    let rest = parts.next().unwrap_or("");

    let val = lookup(args, var).ok_or_else(|| MessageError::UnknownVariable(var.to_string()))?;

    let cases = parse_cases(rest)?;

    match kind {
        "plural" => {
            let n = val
                .as_u64()
                .ok_or_else(|| MessageError::MalformedSelector(var.to_string()))?;
            let cat = plural_category(locale, n).as_str();
            let chosen = cases
                .iter()
                .find(|(k, _)| k == cat)
                .or_else(|| cases.iter().find(|(k, _)| k == "other"))
                .or_else(|| cases.iter().find(|(k, _)| k == "*"))
                .ok_or_else(|| MessageError::MalformedSelector(rest.to_string()))?;
            // Support `#` as shorthand for the numeric value inside the case.
            let rendered = format_depth(locale, &chosen.1, args, depth + 1)?;
            Ok(rendered.replace('#', &n.to_string()))
        }
        "selectordinal" => {
            let n = val
                .as_u64()
                .ok_or_else(|| MessageError::MalformedSelector(var.to_string()))?;
            let cat = ordinal_category(locale, n).as_str();
            let chosen = cases
                .iter()
                .find(|(k, _)| k == cat)
                .or_else(|| cases.iter().find(|(k, _)| k == "other"))
                .or_else(|| cases.iter().find(|(k, _)| k == "*"))
                .ok_or_else(|| MessageError::MalformedSelector(rest.to_string()))?;
            let rendered = format_depth(locale, &chosen.1, args, depth + 1)?;
            Ok(rendered.replace('#', &n.to_string()))
        }
        "select" => {
            let key = val.to_display();
            let chosen = cases
                .iter()
                .find(|(k, _)| *k == key)
                .or_else(|| cases.iter().find(|(k, _)| k == "*"))
                .or_else(|| cases.iter().find(|(k, _)| k == "other"))
                .ok_or_else(|| MessageError::MalformedSelector(rest.to_string()))?;
            format_depth(locale, &chosen.1, args, depth + 1)
        }
        _ => Err(MessageError::MalformedSelector(kind.to_string())),
    }
}

/// Parse `one{...} few{...} other{...}` into `(key, body)` pairs.
fn parse_cases(s: &str) -> Result<Vec<(String, String)>, MessageError> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut cases = Vec::new();
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        let key_start = i;
        while i < chars.len() && chars[i] != '{' {
            i += 1;
        }
        let key: String = chars[key_start..i]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
        if i >= chars.len() {
            return Err(MessageError::UnbalancedBraces);
        }
        let (body, next) = read_block(&chars, i)?;
        cases.push((key, body));
        i = next;
    }
    Ok(cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_placeholder() {
        let args: [(&str, ArgValue); 1] = [("name", ArgValue::Str("Lombok".into()))];
        let out = format("en", "Halo, {name}!", &args).unwrap();
        assert_eq!(out, "Halo, Lombok!");
    }

    #[test]
    fn plural_english() {
        let pattern = "{count, plural, one{# file} other{# files}}";
        let one: [(&str, ArgValue); 1] = [("count", ArgValue::UInt(1))];
        let many: [(&str, ArgValue); 1] = [("count", ArgValue::UInt(5))];
        assert_eq!(format("en", pattern, &one).unwrap(), "1 file");
        assert_eq!(format("en", pattern, &many).unwrap(), "5 files");
    }

    #[test]
    fn plural_indonesian_always_other() {
        let pattern = "{count, plural, one{# berkas} other{# berkas}}";
        let one: [(&str, ArgValue); 1] = [("count", ArgValue::UInt(1))];
        assert_eq!(format("id", pattern, &one).unwrap(), "1 berkas");
    }

    #[test]
    fn selectordinal_english() {
        let p = "{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}";
        let f = |n: u64| format("en", p, &[("n", ArgValue::UInt(n))]).unwrap();
        assert_eq!(
            (f(1).as_str(), f(2).as_str(), f(3).as_str(), f(4).as_str()),
            ("1st", "2nd", "3rd", "4th")
        );
        assert_eq!(
            (f(11).as_str(), f(22).as_str(), f(103).as_str()),
            ("11th", "22nd", "103rd")
        );
        assert_eq!(format("id", p, &[("n", ArgValue::UInt(1))]).unwrap(), "1th");
    }

    #[test]
    fn select_gender() {
        let pattern = "{gender, select, male{Dia (L)} female{Dia (P)} *{Dia}}";
        let male: [(&str, ArgValue); 1] = [("gender", ArgValue::Str("male".into()))];
        let other: [(&str, ArgValue); 1] = [("gender", ArgValue::Str("nonbinary".into()))];
        assert_eq!(format("id", pattern, &male).unwrap(), "Dia (L)");
        assert_eq!(format("id", pattern, &other).unwrap(), "Dia");
    }

    #[test]
    fn nested_plural_with_prefix_suffix() {
        let pattern = "Anda punya {count, plural, one{# pesan baru} other{# pesan baru}}.";
        let args: [(&str, ArgValue); 1] = [("count", ArgValue::UInt(3))];
        assert_eq!(
            format("id", pattern, &args).unwrap(),
            "Anda punya 3 pesan baru."
        );
    }

    #[test]
    fn deep_nesting_is_rejected_not_overflowed() {
        let mut p = String::from("x");
        for _ in 0..200 {
            p = alloc::format!("{{n, select, *{{{}}}}}", p);
        }
        let r = format("en", &p, &[("n", ArgValue::UInt(1))]);
        assert_eq!(r, Err(MessageError::TooDeep));
    }

    #[test]
    fn unknown_variable_errors() {
        let args: [(&str, ArgValue); 0] = [];
        assert_eq!(
            format("en", "Hi {name}", &args),
            Err(MessageError::UnknownVariable("name".to_string()))
        );
    }

    #[test]
    fn unbalanced_braces_errors() {
        let args: [(&str, ArgValue); 0] = [];
        assert_eq!(
            format("en", "Hi {name", &args),
            Err(MessageError::UnbalancedBraces)
        );
    }
}
