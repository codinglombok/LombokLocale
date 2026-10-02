//! Every catalog in locales/ parses and matches the keys and placeholders of `en`.
use lomboklocale::parse_catalog;
use std::collections::BTreeSet;

fn placeholders(s: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = s;
    while let Some(i) = rest.find('{') {
        let tail = &rest[i + 1..];
        let end = tail
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(tail.len());
        out.insert(tail[..end].to_string());
        rest = &tail[end..];
    }
    out
}

#[test]
fn catalogs_match_english() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../locales");
    let read = |loc: &str| {
        parse_catalog(&std::fs::read_to_string(format!("{root}/{loc}/lomboklocale.json")).unwrap())
            .unwrap()
    };
    let en = read("en");
    assert!(!en.is_empty());
    let mut seen = 0;
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let loc = entry.file_name().into_string().unwrap();
        let cat = read(&loc);
        assert_eq!(
            cat.iter().map(|(k, _)| k).collect::<Vec<_>>(),
            en.iter().map(|(k, _)| k).collect::<Vec<_>>(),
            "{loc}"
        );
        for (k, v) in cat.iter() {
            assert_eq!(
                placeholders(v),
                placeholders(en.get(k).unwrap()),
                "{loc}/{k}"
            );
        }
        seen += 1;
    }
    assert!(seen >= 2);
}
