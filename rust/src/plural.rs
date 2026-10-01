//! CLDR plural rules (cardinal), covering Core-20 + Nusantara languages
//! from ARCHITECTURE_UTAMA_v3.3 §7. Each locale implements the subset of
//! `PluralCategory` its CLDR rule set actually uses.

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
    pub fn as_str(&self) -> &'static str {
        match self {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        }
    }
}

/// Cardinal plural category for `n` in `locale`, per CLDR plural rules.
/// Unrecognized locales fall back to the base language subtag, then to
/// the "other-only" family (matches most East/Southeast Asian languages
/// including `id`, `ja`, `zh`, `ko`, `vi`, `th`).
pub fn plural_category(locale: &str, n: u64) -> PluralCategory {
    let lang = locale.split(['-', '_']).next().unwrap_or(locale);
    match lang {
        // English/Germanic family: 1 -> one, else other.
        "en" | "de" | "nl" | "sv" | "da" | "no" | "it" | "el" | "fi" | "hu" | "tr" | "et" => {
            if n == 1 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
        // French/Portuguese-Brazilian family: 0 and 1 -> one.
        "fr" | "pt" => {
            if n == 0 || n == 1 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
        // Russian/Slavic (simplified: one/few/many/other by n mod 10/100).
        "ru" | "uk" | "pl" | "sr" | "hr" | "bs" => {
            let mod10 = n % 10;
            let mod100 = n % 100;
            if mod10 == 1 && mod100 != 11 {
                PluralCategory::One
            } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
                PluralCategory::Few
            } else {
                PluralCategory::Many
            }
        }
        // Arabic: full six-category rule.
        "ar" => {
            let mod100 = n % 100;
            if n == 0 {
                PluralCategory::Zero
            } else if n == 1 {
                PluralCategory::One
            } else if n == 2 {
                PluralCategory::Two
            } else if (3..=10).contains(&mod100) {
                PluralCategory::Few
            } else if (11..=99).contains(&mod100) {
                PluralCategory::Many
            } else {
                PluralCategory::Other
            }
        }
        // Nusantara + CJK + most isolating languages: no plural inflection.
        "id" | "jv" | "su" | "ms" | "ja" | "zh" | "ko" | "vi" | "th" | "km" | "lo" | "my" => {
            PluralCategory::Other
        }
        // Welsh (kept as an example of a richer rule for reference).
        "cy" => match n {
            0 => PluralCategory::Zero,
            1 => PluralCategory::One,
            2 => PluralCategory::Two,
            3 => PluralCategory::Few,
            6 => PluralCategory::Many,
            _ => PluralCategory::Other,
        },
        _ => {
            if n == 1 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
    }
}

/// Ordinal plural category for `n` in `locale` (1st, 2nd, 3rd, 4th …).
/// Covers English, French, Italian, Swedish and Hungarian; languages whose
/// CLDR ordinal rule is "other only" (id, ms, ja, zh, ko, vi, th, de, es, pt,
/// ru, tr, ar, …) and unknown locales return `Other`.
pub fn ordinal_category(locale: &str, n: u64) -> PluralCategory {
    let lang = locale.split(['-', '_']).next().unwrap_or(locale);
    let (m10, m100) = (n % 10, n % 100);
    match lang {
        "en" => {
            if m10 == 1 && m100 != 11 {
                PluralCategory::One
            } else if m10 == 2 && m100 != 12 {
                PluralCategory::Two
            } else if m10 == 3 && m100 != 13 {
                PluralCategory::Few
            } else {
                PluralCategory::Other
            }
        }
        "fr" => {
            if n == 1 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
        "it" => {
            if matches!(n, 11 | 8 | 80 | 800) {
                PluralCategory::Many
            } else {
                PluralCategory::Other
            }
        }
        "sv" => {
            if (m10 == 1 || m10 == 2) && m100 != 11 && m100 != 12 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
        "hu" => {
            if n == 1 || n == 5 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
        _ => PluralCategory::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_one_other() {
        assert_eq!(plural_category("en", 1), PluralCategory::One);
        assert_eq!(plural_category("en-US", 0), PluralCategory::Other);
        assert_eq!(plural_category("en", 2), PluralCategory::Other);
    }

    #[test]
    fn indonesian_has_no_plural() {
        assert_eq!(plural_category("id", 1), PluralCategory::Other);
        assert_eq!(plural_category("id-ID", 100), PluralCategory::Other);
    }

    #[test]
    fn russian_one_few_many() {
        assert_eq!(plural_category("ru", 1), PluralCategory::One);
        assert_eq!(plural_category("ru", 21), PluralCategory::One);
        assert_eq!(plural_category("ru", 2), PluralCategory::Few);
        assert_eq!(plural_category("ru", 5), PluralCategory::Many);
        assert_eq!(plural_category("ru", 11), PluralCategory::Many);
    }

    #[test]
    fn arabic_full_set() {
        assert_eq!(plural_category("ar", 0), PluralCategory::Zero);
        assert_eq!(plural_category("ar", 1), PluralCategory::One);
        assert_eq!(plural_category("ar", 2), PluralCategory::Two);
        assert_eq!(plural_category("ar", 5), PluralCategory::Few);
        assert_eq!(plural_category("ar", 15), PluralCategory::Many);
        assert_eq!(plural_category("ar", 100), PluralCategory::Other);
    }

    #[test]
    fn english_ordinals() {
        let cat = |n| ordinal_category("en", n).as_str();
        assert_eq!(
            (cat(1), cat(2), cat(3), cat(4)),
            ("one", "two", "few", "other")
        );
        assert_eq!((cat(11), cat(12), cat(13)), ("other", "other", "other"));
        assert_eq!(
            (cat(21), cat(22), cat(23), cat(101), cat(111)),
            ("one", "two", "few", "one", "other")
        );
    }

    #[test]
    fn other_language_ordinals() {
        assert_eq!(ordinal_category("fr", 1), PluralCategory::One);
        assert_eq!(ordinal_category("fr", 2), PluralCategory::Other);
        assert_eq!(ordinal_category("it", 8), PluralCategory::Many);
        assert_eq!(ordinal_category("sv", 22), PluralCategory::One);
        assert_eq!(ordinal_category("id", 1), PluralCategory::Other);
    }

    #[test]
    fn french_zero_is_one() {
        assert_eq!(plural_category("fr", 0), PluralCategory::One);
        assert_eq!(plural_category("fr", 1), PluralCategory::One);
        assert_eq!(plural_category("fr", 2), PluralCategory::Other);
    }
}
