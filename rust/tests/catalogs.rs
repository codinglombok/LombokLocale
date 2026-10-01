//! Every locale catalog must have exactly the keys and placeholders of `en`.
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn placeholders(s: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = s;
    while let Some(i) = rest.find('{') {
        if let Some(j) = rest[i..].find('}') {
            out.insert(rest[i + 1..i + j].to_string());
            rest = &rest[i + j + 1..];
        } else {
            break;
        }
    }
    out
}

#[test]
fn all_catalogs_match_english_keys_and_placeholders() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../locales");
    let en = lomboklocale::parse_catalog(
        &fs::read_to_string(root.join("en/lomboklocale.json")).unwrap(),
    )
    .unwrap();
    let mut langs = 0;
    for entry in fs::read_dir(&root).unwrap() {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        let file = dir.join("lomboklocale.json");
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let cat = lomboklocale::parse_catalog(
            &fs::read_to_string(&file).unwrap_or_else(|_| panic!("missing {file:?}")),
        )
        .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(cat.len(), en.len(), "{name}: key count differs from en");
        for key in en_keys(&root) {
            let text = cat
                .get(&key)
                .unwrap_or_else(|| panic!("{name}: missing key {key}"));
            let en_text = en.get(&key).unwrap();
            assert_eq!(
                placeholders(text),
                placeholders(en_text),
                "{name}: placeholders differ for {key}"
            );
            assert!(!text.trim().is_empty(), "{name}: empty {key}");
        }
        langs += 1;
    }
    assert!(langs >= 10, "expected >=10 locales, found {langs}");
}

// Key list is read from the raw en file so this test needs no catalog iterator API.
fn en_keys(root: &Path) -> Vec<String> {
    let raw = fs::read_to_string(root.join("en/lomboklocale.json")).unwrap();
    let mut keys = Vec::new();
    for line in raw.lines() {
        let t = line.trim();
        if let Some(body) = t.strip_prefix('"') {
            if let Some(end) = body.find('"') {
                keys.push(body[..end].to_string());
            }
        }
    }
    keys
}
