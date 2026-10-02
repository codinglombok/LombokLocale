//! API behaviour not covered by the shared vectors.
use lomboklocale::*;

#[test]
fn tags_expose_parts_and_canonical_form() {
    let t = parse_tag("sl-Latn-IT-rozaj-u-ca-gregory-x-priv").unwrap();
    assert_eq!(t.language, "sl");
    assert_eq!(t.script.as_deref(), Some("Latn"));
    assert_eq!(t.region.as_deref(), Some("IT"));
    assert_eq!(t.variants, vec!["rozaj"]);
    assert_eq!(t.extensions, vec![vec!["u", "ca", "gregory"]]);
    assert_eq!(t.private_use, vec!["priv"]);
    assert_eq!(t.base(), "sl-Latn-IT-rozaj");
    assert_eq!(
        t.lookup_chain(),
        vec!["sl-Latn-IT-rozaj", "sl-Latn-IT", "sl-Latn", "sl"]
    );
    assert_eq!(t.to_string(), "sl-Latn-IT-rozaj-u-ca-gregory-x-priv");
}

#[test]
fn errors_carry_code_and_detail() {
    let e = parse_tag("en-$").unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidSubtag);
    assert_eq!(e.detail, "$");
    assert_eq!(e.to_string(), "INVALID_SUBTAG: $");
    assert_eq!(parse_tag("").unwrap_err().to_string(), "EMPTY");
    let all = [
        ErrorCode::Empty,
        ErrorCode::InvalidSubtag,
        ErrorCode::DuplicateVariant,
        ErrorCode::DuplicateExtension,
        ErrorCode::BadNumber,
        ErrorCode::BadOption,
        ErrorCode::BadDate,
        ErrorCode::Syntax,
        ErrorCode::MissingOther,
        ErrorCode::MissingArgument,
        ErrorCode::BadArgument,
        ErrorCode::TooDeep,
        ErrorCode::BadJson,
        ErrorCode::NotObject,
        ErrorCode::NonStringValue,
        ErrorCode::DuplicateKey,
    ];
    let names: Vec<&str> = all.iter().map(|c| c.as_str()).collect();
    assert_eq!(names.len(), 16);
    assert!(names
        .iter()
        .all(|n| n.chars().all(|c| c.is_ascii_uppercase() || c == '_')));
    let boxed: Box<dyn std::error::Error> = Box::new(e);
    assert!(boxed.to_string().starts_with("INVALID_SUBTAG"));
}

#[test]
fn plural_categories_have_names() {
    let names: Vec<&str> = [
        PluralCategory::Zero,
        PluralCategory::One,
        PluralCategory::Two,
        PluralCategory::Few,
        PluralCategory::Many,
        PluralCategory::Other,
    ]
    .iter()
    .map(|c| c.as_str())
    .collect();
    assert_eq!(names, ["zero", "one", "two", "few", "many", "other"]);
    assert_eq!(plural_category("pl", "21").unwrap(), PluralCategory::Many);
    assert_eq!(
        plural_category("en", "NaN").unwrap_err().code,
        ErrorCode::BadNumber
    );
    assert_eq!(
        ordinal_category("en", "x").unwrap_err().code,
        ErrorCode::BadNumber
    );
    // Operands w, t and v with a modulus reach rules of other locales.
    assert_eq!(plural_category("lv", "0.1").unwrap(), PluralCategory::One);
    assert_eq!(
        plural_category("lv", "0.10").unwrap(),
        PluralCategory::Other
    );
    assert_eq!(plural_category("mk", "1.1").unwrap(), PluralCategory::One);
}

#[test]
fn number_options_builders() {
    assert_eq!(
        format_number("en", "1234.5", &NumberOptions::decimal()).unwrap(),
        "1,234.5"
    );
    assert_eq!(
        format_number("en", "0.5", &NumberOptions::percent()).unwrap(),
        "50%"
    );
    assert_eq!(
        format_number("en", "5", &NumberOptions::currency("EUR")).unwrap(),
        "€5.00"
    );
    let opts = NumberOptions::decimal().fraction(Some(2), Some(2));
    assert_eq!(format_number("de", "1", &opts).unwrap(), "1,00");
    assert_eq!(NumberOptions::default().style, Style::Decimal);
}

#[test]
fn dates_parse_and_weekday() {
    let d = Date::parse("2026-10-02").unwrap();
    assert_eq!((d.year, d.month, d.day), (2026, 10, 2));
    assert_eq!(d.weekday(), 5);
    assert_eq!(
        Date::parse("2026-1x-02").unwrap_err().code,
        ErrorCode::BadDate
    );
    assert_eq!(DateStyle::default(), DateStyle::Medium);
    assert_eq!(DateStyle::parse("short"), Some(DateStyle::Short));
    assert_eq!(DateStyle::parse("x"), None);
}

#[test]
fn args_convert_from_rust_values() {
    let args = [
        ("a", Arg::from(-3i64)),
        ("b", Arg::from(7u64)),
        ("c", Arg::from(0.1 + 0.2)),
        ("d", Arg::from("text")),
        ("e", Arg::from(f64::NAN)),
        ("f", Arg::from(f64::INFINITY)),
        ("g", Arg::from(f64::NEG_INFINITY)),
    ];
    assert_eq!(
        format_message("en", "{a} {b} {c} {d} {e} {f} {g}", &args).unwrap(),
        "-3 7 0.3 text NaN ∞ -∞"
    );
    assert_eq!(Arg::str("x"), Arg::Str("x".into()));
    assert_eq!(Arg::num("1.5"), Arg::Num("1.5".into()));
}

#[test]
fn catalogs_and_resolve() {
    let mut c = parse_catalog(r#"{"b": "2", "a": "1"}"#).unwrap();
    assert_eq!(c.len(), 2);
    assert!(!c.is_empty());
    assert_eq!(c.iter().collect::<Vec<_>>(), vec![("a", "1"), ("b", "2")]);
    c.insert("c", "{n, plural, other{# hal}}");
    let m = resolve("id", &c, "C", "c", &[("n", Arg::from(2000u64))]);
    assert_eq!(m.text, "2.000 hal");
    assert!(!m.fallback);
    assert!(Catalog::default().is_empty());
    for bad in [
        r#"{"a": tru}"#,
        r#"{"a": -}"#,
        r#"{"a": 1.}"#,
        r#"{"a": 1e}"#,
        r#"{"a": "\x"}"#,
        r#"{"a": "\u12"}"#,
        r#"{"a" "b"}"#,
        r#"{"a": "\udc00"}"#,
        r#"{"a": "\ud800A"}"#,
        r#"{"a": [1 2]}"#,
    ] {
        assert_eq!(
            parse_catalog(bad).unwrap_err().code,
            ErrorCode::BadJson,
            "{bad}"
        );
    }
    assert_eq!(
        parse_catalog(r#"{"a": {}, "b": []}"#).unwrap_err().code,
        ErrorCode::NonStringValue
    );
    assert_eq!(
        parse_catalog(r#"{"a": "\b\f\r\/\"\\"}"#).unwrap().get("a"),
        Some("\u{8}\u{c}\r/\"\\")
    );
    let deep = format!("{{\"a\": {}{}}}", "[".repeat(100), "]".repeat(100));
    assert_eq!(parse_catalog(&deep).unwrap_err().code, ErrorCode::BadJson);
}

#[test]
fn constants() {
    assert_eq!(CLDR_VERSION, "47.0.0");
    assert_eq!(MAX_DEPTH, 16);
    assert_eq!(MAX_EXPONENT, 1000);
}
