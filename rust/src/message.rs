//! ICU MessageFormat subset (SPEC section 7).

use crate::date::{format_date, DateStyle};
use crate::decimal::{self, Dec};
use crate::error::{Error, ErrorCode, Result};
use crate::number::{format_number, NumberOptions};
use crate::plural::{ordinal_category, plural_category};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Maximum nesting of sub-messages.
pub const MAX_DEPTH: usize = 16;

/// A message argument: text, or a number as a decimal string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arg {
    Str(String),
    Num(String),
}

impl Arg {
    /// Text argument.
    pub fn str(s: impl Into<String>) -> Self {
        Self::Str(s.into())
    }

    /// Number argument from a decimal string such as `"1.50"`.
    pub fn num(s: impl Into<String>) -> Self {
        Self::Num(s.into())
    }
}

impl From<i64> for Arg {
    fn from(v: i64) -> Self {
        Self::Num(v.to_string())
    }
}

impl From<u64> for Arg {
    fn from(v: u64) -> Self {
        Self::Num(v.to_string())
    }
}

impl From<f64> for Arg {
    /// NaN and infinities become `NaN`, `Infinity`, `-Infinity`; other values
    /// use the shortest decimal that reads back as the same `f64`.
    fn from(v: f64) -> Self {
        Self::Num(if v.is_nan() {
            "NaN".to_string()
        } else if v.is_infinite() {
            if v < 0.0 { "-Infinity" } else { "Infinity" }.to_string()
        } else {
            alloc::format!("{}", v)
        })
    }
}

impl From<&str> for Arg {
    fn from(v: &str) -> Self {
        Self::Str(v.to_string())
    }
}

/// Named arguments.
pub type Args<'a> = &'a [(&'a str, Arg)];

#[derive(Debug)]
enum Part {
    Text(String),
    Pound,
    Simple(String),
    Number(String, Option<String>),
    Date(String, DateStyle),
    Choice {
        kind: Kind,
        name: String,
        offset: u64,
        cases: Vec<(String, Vec<Part>)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Plural,
    Ordinal,
    Select,
}

struct Parser<'a> {
    s: &'a [char],
    i: usize,
}

fn syntax(i: usize) -> Error {
    Error::new(ErrorCode::Syntax, alloc::format!("{}", i))
}

fn is_name(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }

    fn message(&mut self, depth: usize, in_plural: bool, top: bool) -> Result<Vec<Part>> {
        if depth > MAX_DEPTH {
            return Err(Error::new(ErrorCode::TooDeep, ""));
        }
        let mut parts = Vec::new();
        let mut buf = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\'' => {
                    let next = self.s.get(self.i + 1).copied();
                    if next == Some('\'') {
                        buf.push('\'');
                        self.i += 2;
                    } else if matches!(next, Some('{' | '}' | '|'))
                        || (next == Some('#') && in_plural)
                    {
                        self.i += 1;
                        while let Some(q) = self.peek() {
                            if q == '\'' {
                                if self.s.get(self.i + 1) == Some(&'\'') {
                                    buf.push('\'');
                                    self.i += 2;
                                    continue;
                                }
                                self.i += 1;
                                break;
                            }
                            buf.push(q);
                            self.i += 1;
                        }
                    } else {
                        buf.push('\'');
                        self.i += 1;
                    }
                }
                '{' => {
                    if !buf.is_empty() {
                        parts.push(Part::Text(core::mem::take(&mut buf)));
                    }
                    self.i += 1;
                    parts.push(self.argument(depth, in_plural)?);
                }
                '}' => {
                    if top {
                        return Err(syntax(self.i));
                    }
                    break;
                }
                '#' if in_plural => {
                    if !buf.is_empty() {
                        parts.push(Part::Text(core::mem::take(&mut buf)));
                    }
                    parts.push(Part::Pound);
                    self.i += 1;
                }
                _ => {
                    buf.push(c);
                    self.i += 1;
                }
            }
        }
        if !buf.is_empty() {
            parts.push(Part::Text(buf));
        }
        Ok(parts)
    }

    fn ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\r' | '\n')) {
            self.i += 1;
        }
    }

    fn word(&mut self) -> String {
        self.ws();
        let start = self.i;
        while let Some(c) = self.peek() {
            if matches!(c, ' ' | '\t' | '\r' | '\n' | ',' | '{' | '}') {
                break;
            }
            self.i += 1;
        }
        self.s[start..self.i].iter().collect()
    }

    fn expect(&mut self, c: char) -> Result<()> {
        self.ws();
        if self.peek() != Some(c) {
            return Err(syntax(self.i));
        }
        self.i += 1;
        Ok(())
    }

    fn argument(&mut self, depth: usize, in_plural: bool) -> Result<Part> {
        let name = self.word();
        if !is_name(&name) {
            return Err(syntax(self.i));
        }
        self.ws();
        match self.peek() {
            None => return Err(syntax(self.i)),
            Some('}') => {
                self.i += 1;
                return Ok(Part::Simple(name));
            }
            _ => {}
        }
        self.expect(',')?;
        let kind = self.word();
        if kind == "number" || kind == "date" {
            self.ws();
            let mut style = None;
            if self.peek() == Some(',') {
                self.i += 1;
                let st = self.word();
                let ok = if kind == "number" {
                    matches!(st.as_str(), "integer" | "percent")
                } else {
                    DateStyle::parse(&st).is_some()
                };
                if !ok {
                    return Err(syntax(self.i));
                }
                style = Some(st);
            }
            self.expect('}')?;
            return Ok(if kind == "number" {
                Part::Number(name, style)
            } else {
                Part::Date(
                    name,
                    style
                        .as_deref()
                        .and_then(DateStyle::parse)
                        .unwrap_or_default(),
                )
            });
        }
        let kind = match kind.as_str() {
            "plural" => Kind::Plural,
            "selectordinal" => Kind::Ordinal,
            "select" => Kind::Select,
            _ => return Err(syntax(self.i)),
        };
        self.expect(',')?;
        let mut offset = 0u64;
        let mut seen_offset = false;
        let mut cases: Vec<(String, Vec<Part>)> = Vec::new();
        loop {
            self.ws();
            match self.peek() {
                None => return Err(syntax(self.i)),
                Some('}') => {
                    self.i += 1;
                    break;
                }
                _ => {}
            }
            let key = self.word();
            if let Some(rest) = key.strip_prefix("offset:") {
                if kind == Kind::Plural && cases.is_empty() && !seen_offset {
                    let rest = if rest.is_empty() {
                        self.word()
                    } else {
                        rest.to_string()
                    };
                    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(syntax(self.i));
                    }
                    offset = rest.parse().map_err(|_| syntax(self.i))?;
                    seen_offset = true;
                    continue;
                }
            }
            if key.is_empty() || cases.iter().any(|(k, _)| *k == key) {
                return Err(syntax(self.i));
            }
            let valid = match kind {
                Kind::Select => is_name(&key),
                _ => match key.strip_prefix('=') {
                    Some(v) => is_exact(v),
                    None => matches!(
                        key.as_str(),
                        "zero" | "one" | "two" | "few" | "many" | "other"
                    ),
                },
            };
            if !valid {
                return Err(syntax(self.i));
            }
            self.expect('{')?;
            let body = self.message(depth + 1, kind != Kind::Select || in_plural, false)?;
            if self.peek() != Some('}') {
                return Err(syntax(self.i));
            }
            self.i += 1;
            cases.push((key, body));
        }
        if !cases.iter().any(|(k, _)| k == "other") {
            return Err(Error::new(ErrorCode::MissingOther, name));
        }
        Ok(Part::Choice {
            kind,
            name,
            offset,
            cases,
        })
    }
}

/// `digits[.digits]`
fn is_exact(v: &str) -> bool {
    let (a, b) = match v.split_once('.') {
        Some((a, b)) => (a, Some(b)),
        None => (v, None),
    };
    !a.is_empty()
        && a.bytes().all(|c| c.is_ascii_digit())
        && b.map_or(true, |b| {
            !b.is_empty() && b.bytes().all(|c| c.is_ascii_digit())
        })
}

fn lookup<'a>(args: Args<'a>, name: &str) -> Result<&'a Arg> {
    args.iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| v)
        .ok_or_else(|| Error::new(ErrorCode::MissingArgument, name))
}

fn finite(v: &str) -> Option<(bool, String, String)> {
    match decimal::parse(v) {
        Ok(Dec::Finite { neg, int, frac }) => Some((neg, int, frac)),
        _ => None,
    }
}

fn is_zero(int: &str, frac: &str) -> bool {
    int == "0" && frac.bytes().all(|b| b == b'0')
}

fn dec_eq(a: &str, b: &str) -> bool {
    match (finite(a), finite(b)) {
        (Some((na, ia, fa)), Some((nb, ib, fb))) => {
            if is_zero(&ia, &fa) && is_zero(&ib, &fb) {
                return true;
            }
            na == nb && ia == ib && fa.trim_end_matches('0') == fb.trim_end_matches('0')
        }
        _ => false,
    }
}

/// `value - offset` as a decimal string keeping the fraction digits; None
/// when the value has more than 36 digits (SPEC section 7.4).
fn dec_sub(value: &str, offset: u64) -> Option<String> {
    if offset == 0 {
        return Some(value.to_string());
    }
    let (neg, int, frac) = finite(value)?;
    let scale = frac.len() as u32;
    let digits = alloc::format!("{}{}", int, frac);
    if digits.len() > 36 {
        return None;
    }
    let mut n: i128 = digits.parse().ok()?;
    if neg {
        n = -n;
    }
    let n = n - i128::from(offset) * 10i128.pow(scale);
    let mut s = alloc::format!("{}", n.unsigned_abs());
    while s.len() < scale as usize + 1 {
        s.insert(0, '0');
    }
    let (a, b) = s.split_at(s.len() - scale as usize);
    let mut out = String::new();
    if n < 0 {
        out.push('-');
    }
    out.push_str(a);
    if scale > 0 {
        out.push('.');
        out.push_str(b);
    }
    Some(out)
}

fn for_plural(value: &str) -> String {
    let (_, int, frac) = finite(value).expect("checked finite");
    let (ip, fp) = decimal::round(&int, &frac, 3);
    let fp = fp.trim_end_matches('0');
    if fp.is_empty() {
        ip
    } else {
        alloc::format!("{}.{}", ip, fp)
    }
}

fn render(
    locale: &str,
    parts: &[Part],
    args: Args,
    pound: Option<&str>,
    out: &mut String,
) -> Result<()> {
    for part in parts {
        match part {
            Part::Text(t) => out.push_str(t),
            Part::Pound => match pound {
                Some(p) => out.push_str(&format_number(locale, p, &NumberOptions::decimal())?),
                None => out.push('#'),
            },
            Part::Simple(name) => match lookup(args, name)? {
                Arg::Str(s) => out.push_str(s),
                Arg::Num(n) => out.push_str(
                    &format_number(locale, n, &NumberOptions::decimal())
                        .map_err(|_| Error::new(ErrorCode::BadArgument, name.clone()))?,
                ),
            },
            Part::Number(name, style) => {
                let Arg::Num(n) = lookup(args, name)? else {
                    return Err(Error::new(ErrorCode::BadArgument, name.clone()));
                };
                let opts = match style.as_deref() {
                    Some("integer") => NumberOptions::decimal().fraction(None, Some(0)),
                    Some(_) => NumberOptions::percent(),
                    None => NumberOptions::decimal(),
                };
                out.push_str(
                    &format_number(locale, n, &opts)
                        .map_err(|_| Error::new(ErrorCode::BadArgument, name.clone()))?,
                );
            }
            Part::Date(name, style) => {
                let Arg::Str(s) = lookup(args, name)? else {
                    return Err(Error::new(ErrorCode::BadArgument, name.clone()));
                };
                out.push_str(
                    &format_date(locale, s, *style)
                        .map_err(|_| Error::new(ErrorCode::BadArgument, name.clone()))?,
                );
            }
            Part::Choice {
                kind,
                name,
                offset,
                cases,
            } => {
                let arg = lookup(args, name)?;
                let other = &cases.iter().find(|(k, _)| k == "other").expect("checked").1;
                if *kind == Kind::Select {
                    let key = match arg {
                        Arg::Str(s) | Arg::Num(s) => s,
                    };
                    let chosen = cases.iter().find(|(k, _)| k == key).map_or(other, |c| &c.1);
                    render(locale, chosen, args, pound, out)?;
                    continue;
                }
                let Arg::Num(n) = arg else {
                    return Err(Error::new(ErrorCode::BadArgument, name.clone()));
                };
                if finite(n).is_none() {
                    return Err(Error::new(ErrorCode::BadArgument, name.clone()));
                }
                let shown = dec_sub(n, *offset)
                    .ok_or_else(|| Error::new(ErrorCode::BadArgument, name.clone()))?;
                let exact = cases
                    .iter()
                    .find(|(k, _)| k.strip_prefix('=').is_some_and(|v| dec_eq(v, n)));
                let chosen = match exact {
                    Some(c) => &c.1,
                    None => {
                        let v = for_plural(&shown);
                        let cat = if *kind == Kind::Ordinal {
                            ordinal_category(locale, &v)?
                        } else {
                            plural_category(locale, &v)?
                        };
                        cases
                            .iter()
                            .find(|(k, _)| k == cat.as_str())
                            .map_or(other, |c| &c.1)
                    }
                };
                render(locale, chosen, args, Some(&shown), out)?;
            }
        }
    }
    Ok(())
}

/// Formats an ICU MessageFormat `pattern` with `args` for `locale`.
///
/// # Errors
/// `SYNTAX`, `MISSING_OTHER`, `TOO_DEEP` for the pattern;
/// `MISSING_ARGUMENT`, `BAD_ARGUMENT` for the arguments.
pub fn format_message(locale: &str, pattern: &str, args: Args) -> Result<String> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut p = Parser { s: &chars, i: 0 };
    let parts = p.message(0, false, true)?;
    let mut out = String::new();
    render(locale, &parts, args, None, &mut out)?;
    Ok(out)
}
