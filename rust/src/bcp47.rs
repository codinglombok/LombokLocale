//! BCP 47 (RFC 5646) language tag parsing and locale negotiation.
//!
//! Supports the common subset used across the Lombok ecosystem:
//! `language[-script][-region][-variant...]`, e.g. `id`, `en-US`, `zh-Hant-TW`.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// A parsed BCP-47 language tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleTag {
    pub language: String,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocaleError {
    Empty,
    InvalidSubtag(String),
}

impl core::fmt::Display for LocaleTag {
    /// Canonical form: `id`, `en-US`, `zh-Hant-TW`.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.language)?;
        if let Some(s) = &self.script {
            write!(f, "-{}", s)?;
        }
        if let Some(r) = &self.region {
            write!(f, "-{}", r)?;
        }
        for v in &self.variants {
            write!(f, "-{}", v)?;
        }
        Ok(())
    }
}

impl LocaleTag {
    /// The `language` subtag alone, e.g. `en-US` -> `en`.
    pub fn base_language(&self) -> &str {
        &self.language
    }
}

fn is_alpha(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic())
}

fn is_alnum(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Parse a BCP-47 tag such as `en`, `en-US`, `zh-Hant-TW`, `id-ID`.
pub fn parse_bcp47(tag: &str) -> Result<LocaleTag, LocaleError> {
    if tag.trim().is_empty() {
        return Err(LocaleError::Empty);
    }
    let parts: Vec<&str> = tag.trim().split(['-', '_']).collect();
    let mut idx = 0;

    let language = parts.first().copied().unwrap_or("");
    if !is_alpha(language) || !(2..=8).contains(&language.len()) {
        return Err(LocaleError::InvalidSubtag(language.to_string()));
    }
    let language = language.to_ascii_lowercase();
    idx += 1;

    let mut script = None;
    if let Some(p) = parts.get(idx) {
        if p.len() == 4 && is_alpha(p) {
            let mut s = p.to_ascii_lowercase();
            // Titlecase: first letter upper, rest lower (ISO 15924 convention).
            if let Some(first) = s.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            script = Some(s);
            idx += 1;
        }
    }

    let mut region = None;
    if let Some(p) = parts.get(idx) {
        if (p.len() == 2 && is_alpha(p)) || (p.len() == 3 && p.chars().all(|c| c.is_ascii_digit()))
        {
            region = Some(p.to_ascii_uppercase());
            idx += 1;
        }
    }

    let mut variants = Vec::new();
    while let Some(p) = parts.get(idx) {
        if is_alnum(p) && p.len() >= 4 {
            variants.push(p.to_ascii_lowercase());
            idx += 1;
        } else {
            return Err(LocaleError::InvalidSubtag(p.to_string()));
        }
    }

    Ok(LocaleTag {
        language,
        script,
        region,
        variants,
    })
}

/// RFC 4647 "lookup" style negotiation: for each requested tag (in priority
/// order), fall back from most to least specific (`en-US` -> `en`) and
/// return the first available match. Falls back to `default` if nothing
/// matches.
pub fn negotiate<'a>(requested: &[&str], available: &[&'a str], default: &'a str) -> String {
    for req in requested {
        let parsed = match parse_bcp47(req) {
            Ok(p) => p,
            Err(_) => continue,
        };
        // RFC 4647 lookup: progressively drop trailing subtags
        // (lang-script-region-variants -> lang-script-region -> lang-script -> lang),
        // then also try lang-region (CLDR-style) before the bare language.
        let mut chain: Vec<String> = Vec::new();
        chain.push(parsed.to_string());
        let mut base = parsed.language.clone();
        if let Some(sc) = &parsed.script {
            base.push('-');
            base.push_str(sc);
        }
        if let Some(rg) = &parsed.region {
            let mut full = base.clone();
            full.push('-');
            full.push_str(rg);
            chain.push(full);
        }
        chain.push(base);
        if parsed.script.is_some() {
            if let Some(rg) = &parsed.region {
                chain.push(alloc::format!("{}-{}", parsed.language, rg));
            }
        }
        chain.push(parsed.language.clone());
        chain.dedup();

        for candidate in &chain {
            for avail in available {
                if avail.eq_ignore_ascii_case(candidate) {
                    return (*avail).to_string();
                }
            }
        }
    }
    default.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple() {
        let t = parse_bcp47("id").unwrap();
        assert_eq!(t.language, "id");
        assert!(t.region.is_none());
        assert_eq!(t.to_string(), "id");
    }

    #[test]
    fn parses_lang_region() {
        let t = parse_bcp47("en-US").unwrap();
        assert_eq!(t.language, "en");
        assert_eq!(t.region.as_deref(), Some("US"));
        assert_eq!(t.to_string(), "en-US");
    }

    #[test]
    fn parses_lang_script_region() {
        let t = parse_bcp47("zh-Hant-TW").unwrap();
        assert_eq!(t.language, "zh");
        assert_eq!(t.script.as_deref(), Some("Hant"));
        assert_eq!(t.region.as_deref(), Some("TW"));
        assert_eq!(t.to_string(), "zh-Hant-TW");
    }

    #[test]
    fn normalizes_case_and_underscore() {
        let t = parse_bcp47("EN_us").unwrap();
        assert_eq!(t.to_string(), "en-US");
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(parse_bcp47(""), Err(LocaleError::Empty));
    }

    #[test]
    fn negotiate_exact_match() {
        let avail = ["en", "id", "ja"];
        assert_eq!(negotiate(&["id-ID"], &avail, "en"), "id");
    }

    #[test]
    fn negotiate_falls_back_to_language() {
        let avail = ["en", "id"];
        assert_eq!(negotiate(&["en-GB"], &avail, "id"), "en");
    }

    #[test]
    fn negotiate_follows_rfc4647_truncation_order() {
        // zh-Hant-TW -> zh-Hant (script kept) before zh-TW before zh
        assert_eq!(
            negotiate(&["zh-Hant-TW"], &["zh", "zh-Hant"], "en"),
            "zh-Hant"
        );
        assert_eq!(negotiate(&["zh-Hant-TW"], &["zh", "zh-TW"], "en"), "zh-TW");
        assert_eq!(
            negotiate(&["zh-Hant-TW"], &["zh-Hant-TW", "zh-Hant"], "en"),
            "zh-Hant-TW"
        );
        assert_eq!(negotiate(&["de-CH-1996"], &["de-CH", "de"], "en"), "de-CH");
    }

    #[test]
    fn negotiate_falls_back_to_default() {
        let avail = ["en", "id"];
        assert_eq!(negotiate(&["fr-FR"], &avail, "en"), "en");
    }

    #[test]
    fn negotiate_tries_priority_list() {
        let avail = ["id", "ja"];
        assert_eq!(negotiate(&["fr", "ja-JP"], &avail, "en"), "ja");
    }
}
