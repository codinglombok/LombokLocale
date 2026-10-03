//! BCP 47 language tags (SPEC section 2) and RFC 4647 lookup.

use crate::error::{Error, ErrorCode, Result};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// A well-formed language tag split into its parts, in canonical case.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LanguageTag {
    pub language: String,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Vec<String>,
    /// Each extension is its singleton followed by its subtags.
    pub extensions: Vec<Vec<String>>,
    pub private_use: Vec<String>,
}

fn is_alpha(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_alphabetic())
}

fn is_digits(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_digit())
}

fn title(s: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let mut out = String::with_capacity(lower.len());
    out.push_str(&lower[..1].to_ascii_uppercase());
    out.push_str(&lower[1..]);
    out
}

/// Parses a tag; `_` is accepted as a separator and surrounding spaces are
/// ignored.
///
/// # Errors
/// `EMPTY`, `INVALID_SUBTAG`, `DUPLICATE_VARIANT`, `DUPLICATE_EXTENSION`.
pub fn parse_tag(tag: &str) -> Result<LanguageTag> {
    let tag = tag.trim();
    if tag.is_empty() {
        return Err(Error::new(ErrorCode::Empty, ""));
    }
    let subs: Vec<&str> = tag.split(['-', '_']).collect();
    for s in &subs {
        if s.is_empty() || s.len() > 8 || !s.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return Err(Error::new(ErrorCode::InvalidSubtag, *s));
        }
    }
    let lang = subs[0];
    if !(is_alpha(lang) && ((2..=3).contains(&lang.len()) || (5..=8).contains(&lang.len()))) {
        return Err(Error::new(ErrorCode::InvalidSubtag, lang));
    }
    let mut out = LanguageTag {
        language: lang.to_ascii_lowercase(),
        ..LanguageTag::default()
    };
    let mut i = 1;
    if i < subs.len() && subs[i].len() == 4 && is_alpha(subs[i]) {
        out.script = Some(title(subs[i]));
        i += 1;
    }
    if i < subs.len()
        && ((subs[i].len() == 2 && is_alpha(subs[i])) || (subs[i].len() == 3 && is_digits(subs[i])))
    {
        out.region = Some(subs[i].to_ascii_uppercase());
        i += 1;
    }
    while i < subs.len()
        && ((5..=8).contains(&subs[i].len())
            || (subs[i].len() == 4 && subs[i].as_bytes()[0].is_ascii_digit()))
    {
        let v = subs[i].to_ascii_lowercase();
        if out.variants.contains(&v) {
            return Err(Error::new(ErrorCode::DuplicateVariant, v));
        }
        out.variants.push(v);
        i += 1;
    }
    while i < subs.len() && subs[i].len() == 1 && !subs[i].eq_ignore_ascii_case("x") {
        let single = subs[i].to_ascii_lowercase();
        if out.extensions.iter().any(|e| e[0] == single) {
            return Err(Error::new(ErrorCode::DuplicateExtension, single));
        }
        i += 1;
        let mut ext = alloc::vec![single.clone()];
        while i < subs.len() && subs[i].len() >= 2 {
            ext.push(subs[i].to_ascii_lowercase());
            i += 1;
        }
        if ext.len() == 1 {
            return Err(Error::new(ErrorCode::InvalidSubtag, single));
        }
        out.extensions.push(ext);
    }
    if i < subs.len() && subs[i].eq_ignore_ascii_case("x") {
        i += 1;
        if i == subs.len() {
            return Err(Error::new(ErrorCode::InvalidSubtag, "x"));
        }
        while i < subs.len() {
            out.private_use.push(subs[i].to_ascii_lowercase());
            i += 1;
        }
    }
    if i < subs.len() {
        return Err(Error::new(ErrorCode::InvalidSubtag, subs[i]));
    }
    out.extensions.sort();
    Ok(out)
}

impl LanguageTag {
    /// language, script, region and variants joined with `-` (no extensions).
    pub fn base(&self) -> String {
        let mut parts: Vec<&str> = alloc::vec![self.language.as_str()];
        if let Some(s) = &self.script {
            parts.push(s);
        }
        if let Some(r) = &self.region {
            parts.push(r);
        }
        for v in &self.variants {
            parts.push(v);
        }
        parts.join("-")
    }

    /// RFC 4647 lookup fallback list: the base tag, then shorter prefixes.
    pub fn lookup_chain(&self) -> Vec<String> {
        let base = self.base();
        let parts: Vec<&str> = base.split('-').collect();
        (1..=parts.len())
            .rev()
            .map(|k| parts[..k].join("-"))
            .collect()
    }
}

impl fmt::Display for LanguageTag {
    /// Canonical form with extensions sorted by singleton.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.base())?;
        for e in &self.extensions {
            for s in e {
                write!(f, "-{}", s)?;
            }
        }
        if !self.private_use.is_empty() {
            f.write_str("-x")?;
            for s in &self.private_use {
                write!(f, "-{}", s)?;
            }
        }
        Ok(())
    }
}

/// Canonical form of `tag`, for example `"EN_us"` -> `"en-US"`.
///
/// # Errors
/// See [`parse_tag`].
pub fn canonicalize(tag: &str) -> Result<String> {
    parse_tag(tag).map(|t| t.to_string())
}

/// RFC 4647 lookup: for each requested tag in order, try the tag without
/// extensions and then each shorter prefix against `available`
/// (case-insensitive); return the first match as written in `available`, or
/// `default`. Requested tags that do not parse are skipped.
pub fn negotiate<S: AsRef<str>, T: AsRef<str>>(
    requested: &[S],
    available: &[T],
    default: &str,
) -> String {
    let avail: Vec<(String, &str)> = available
        .iter()
        .filter_map(|a| {
            parse_tag(a.as_ref())
                .ok()
                .map(|t| (t.base().to_ascii_lowercase(), a.as_ref()))
        })
        .collect();
    for r in requested {
        let Ok(t) = parse_tag(r.as_ref()) else {
            continue;
        };
        for cand in t.lookup_chain() {
            let cand = cand.to_ascii_lowercase();
            if let Some((_, orig)) = avail.iter().find(|(c, _)| *c == cand) {
                return (*orig).to_string();
            }
        }
    }
    default.to_string()
}

/// The CLDR data locale used for formatting `tag` (SPEC section 2.3): Chinese
/// in TW, HK or MO without a script uses `zh-Hant`; otherwise the lookup chain
/// is searched; `en` when nothing matches or the tag is invalid.
pub fn data_locale(tag: &str) -> &'static str {
    let Ok(mut t) = parse_tag(tag) else {
        return "en";
    };
    if t.language == "zh"
        && t.script.is_none()
        && matches!(t.region.as_deref(), Some("TW" | "HK" | "MO"))
    {
        t.script = Some("Hant".to_string());
    }
    for cand in t.lookup_chain() {
        if let Some((name, _)) = crate::data::LOCALES.iter().find(|(n, _)| *n == cand) {
            return name;
        }
    }
    "en"
}

pub(crate) fn locale_data(tag: &str) -> &'static crate::tables::LocaleData {
    let name = data_locale(tag);
    &crate::data::LOCALES
        .iter()
        .find(|(n, _)| *n == name)
        .expect("data locale")
        .1
}
