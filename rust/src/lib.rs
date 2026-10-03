//! LombokLocale: BCP 47 tags and negotiation, CLDR 47 plural rules,
//! number/percent/currency and date formatting, an ICU MessageFormat subset,
//! and JSON message catalogs. The same input gives the same output in Rust,
//! TypeScript, Python, Go and PHP (see `docs/SPEC_LombokLocale_v0.2.0.md`).
//!
//! `no_std` + `alloc` without the default `std` feature. No dependencies.
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod catalog;
#[rustfmt::skip]
mod data;
mod date;
mod decimal;
mod error;
mod message;
mod number;
mod plural;
mod tables;
mod tag;

pub use catalog::{parse_catalog, Catalog};
pub use date::{format_date, Date, DateStyle};
pub use error::{Error, ErrorCode, Result};
pub use message::{format_message, Arg, Args, MAX_DEPTH};
pub use number::{format_number, NumberOptions, Style};
pub use plural::{ordinal_category, plural_category, PluralCategory};
pub use tag::{canonicalize, data_locale, negotiate, parse_tag, LanguageTag};

/// CLDR release of the embedded data.
pub const CLDR_VERSION: &str = data::CLDR_VERSION;
/// Largest exponent magnitude accepted in decimal strings.
pub const MAX_EXPONENT: i64 = decimal::MAX_EXPONENT;

/// A catalog-backed message: the stable `code` callers match on, the
/// `message_id` used for lookup, and the text shown to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalizedMessage {
    pub code: alloc::string::String,
    pub message_id: alloc::string::String,
    pub text: alloc::string::String,
    /// True when the id was missing or its pattern failed; `text` is then `!!id!!`.
    pub fallback: bool,
}

/// Looks up `message_id` in `catalog` and formats it. A missing id or a
/// pattern that fails to format yields `!!message_id!!` with `fallback` set,
/// so a missing translation is visible instead of blank.
pub fn resolve(
    locale: &str,
    catalog: &Catalog,
    code: &str,
    message_id: &str,
    args: Args,
) -> LocalizedMessage {
    let text = catalog
        .get(message_id)
        .and_then(|p| format_message(locale, p, args).ok());
    LocalizedMessage {
        code: code.into(),
        message_id: message_id.into(),
        fallback: text.is_none(),
        text: text.unwrap_or_else(|| alloc::format!("!!{}!!", message_id)),
    }
}
