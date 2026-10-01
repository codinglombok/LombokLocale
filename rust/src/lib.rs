//! LombokLocale — zero-dependency i18n core (L0).
//!
//! Implements ARCHITECTURE_UTAMA_v3.3 §7: BCP-47 negotiation, a CLDR
//! plural-rule subset, a MessageFormat 2-lite runtime, and locale-aware
//! number/date/currency formatting, backed by flat JSON message catalogs
//! (`locales/<bcp47>/<repo>.json`).
//!
//! `no_std + alloc` by default; enable the `std` feature (on by default
//! via Cargo.toml) for convenience re-exports where relevant. No
//! mandatory dependency on any other Lombok library — L0 per ADR-016's
//! sibling rule that L0 repos carry zero mandatory Lombok deps.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod bcp47;
pub mod catalog;
pub mod date;
pub mod message;
pub mod number;

pub use bcp47::{negotiate, parse_bcp47, LocaleError, LocaleTag};
pub use catalog::{parse_catalog, Catalog, CatalogError};
pub use date::{format_date, parse_iso_date, DateStyle, SimpleDate};
pub use message::{format as format_message, ArgValue, MessageError};
pub use number::{format_currency, format_float, format_integer, MAX_DECIMALS};
pub use plural::{ordinal_category, plural_category, PluralCategory};

pub mod plural;

/// A translated, catalog-backed message. Carries both the stable `code`
/// (the contract other repos match against) and the `message_id` used to
/// look up localized text — matching the "error membawa `code` (kontrak)
/// dan `messageId` (terjemahan)" convention in ARCHITECTURE_UTAMA §7.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalizedMessage {
    pub code: alloc::string::String,
    pub message_id: alloc::string::String,
    pub text: alloc::string::String,
}

/// Resolve `message_id` from `catalog`, formatting it with `args`. Falls
/// back to the raw `message_id` (wrapped in `!! !!`) if the catalog has no
/// entry, so a missing translation is visibly wrong rather than silently
/// blank in the UI.
pub fn resolve(
    locale: &str,
    catalog: &Catalog,
    code: &str,
    message_id: &str,
    args: message::Args,
) -> LocalizedMessage {
    let pattern = catalog.get(message_id);
    let text = match pattern {
        Some(p) => message::format(locale, p, args)
            .unwrap_or_else(|_| alloc::format!("!!{}!!", message_id)),
        None => alloc::format!("!!{}!!", message_id),
    };
    LocalizedMessage {
        code: code.into(),
        message_id: message_id.into(),
        text,
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn end_to_end_catalog_and_message() {
        let json = r#"{"validator.too_short": "Minimal {min} karakter"}"#;
        let cat = parse_catalog(json).unwrap();
        let args: [(&str, ArgValue); 1] = [("min", ArgValue::UInt(8))];
        let msg = resolve("id", &cat, "TOO_SHORT", "validator.too_short", &args);
        assert_eq!(msg.text, "Minimal 8 karakter");
        assert_eq!(msg.code, "TOO_SHORT");
    }

    #[test]
    fn missing_message_id_is_visibly_marked() {
        let cat = Catalog::default();
        let args: [(&str, ArgValue); 0] = [];
        let msg = resolve("id", &cat, "X", "nonexistent.id", &args);
        assert_eq!(msg.text, "!!nonexistent.id!!");
    }

    #[test]
    fn negotiate_then_load_catalog() {
        let available = ["en", "id"];
        let picked = negotiate(&["id-ID", "en"], &available, "en");
        assert_eq!(picked, "id");
    }
}
