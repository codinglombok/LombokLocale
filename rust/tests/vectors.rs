//! Runs every case of vectors/lomboklocale-vectors-v1.json.
use lomboklocale::*;
use serde_json::{json, Map, Value};

fn args(v: &Value) -> Vec<(String, Arg)> {
    v.as_object()
        .unwrap()
        .iter()
        .map(|(k, a)| {
            let arg = match a.get("s") {
                Some(s) => Arg::str(s.as_str().unwrap()),
                None => Arg::num(a["n"].as_str().unwrap()),
            };
            (k.clone(), arg)
        })
        .collect()
}

fn ok<T: Into<Value>>(r: Result<T>) -> Value {
    match r {
        Ok(v) => json!({ "ok": v.into() }),
        Err(e) => json!({ "error": e.code.as_str() }),
    }
}

fn run(kind: &str, i: &Value) -> Value {
    let s = |k: &str| i[k].as_str().unwrap();
    let strs = |k: &str| {
        i[k].as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>()
    };
    match kind {
        "parse" => ok(canonicalize(s("tag"))),
        "negotiate" => {
            json!({ "ok": negotiate(&strs("requested"), &strs("available"), s("default")) })
        }
        "dataLocale" => json!({ "ok": data_locale(s("tag")) }),
        "plural" => {
            let r = if i.get("ordinal").and_then(Value::as_bool).unwrap_or(false) {
                ordinal_category(s("locale"), s("value"))
            } else {
                plural_category(s("locale"), s("value"))
            };
            ok(r.map(|c| c.as_str()))
        }
        "number" => {
            let style = match i.get("style").and_then(Value::as_str).unwrap_or("decimal") {
                "decimal" => Some(Style::Decimal),
                "percent" => Some(Style::Percent),
                "currency" => i
                    .get("currency")
                    .and_then(Value::as_str)
                    .map(|c| Style::Currency(c.into())),
                _ => None,
            };
            let Some(style) = style else {
                return json!({ "error": "BAD_OPTION" });
            };
            let frac = |k: &str| i.get(k).and_then(Value::as_u64).map(|v| v.min(255) as u8);
            let opts = NumberOptions {
                style,
                min_fraction: frac("minFraction"),
                max_fraction: frac("maxFraction"),
            };
            ok(format_number(s("locale"), s("value"), &opts))
        }
        "date" => match DateStyle::parse(s("style")) {
            Some(st) => ok(format_date(s("locale"), s("date"), st)),
            None => json!({ "error": "BAD_OPTION" }),
        },
        "message" => {
            let a = args(&i["args"]);
            let refs: Vec<(&str, Arg)> = a.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
            ok(format_message(s("locale"), s("pattern"), &refs))
        }
        "catalog" => ok(parse_catalog(s("json")).map(|c| {
            let mut m = Map::new();
            for (k, v) in c.iter() {
                m.insert(k.into(), v.into());
            }
            Value::Object(m)
        })),
        "resolve" => {
            let cat = parse_catalog(s("catalog")).unwrap();
            let a = args(&i["args"]);
            let refs: Vec<(&str, Arg)> = a.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
            let m = resolve(s("locale"), &cat, s("code"), s("messageId"), &refs);
            json!({ "ok": { "code": m.code, "messageId": m.message_id, "text": m.text, "fallback": m.fallback } })
        }
        _ => panic!("unknown kind {kind}"),
    }
}

#[test]
fn shared_vectors() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../vectors/lomboklocale-vectors-v1.json"
    ))
    .unwrap();
    let doc: Value = serde_json::from_str(&text).unwrap();
    let cases = doc["cases"].as_array().unwrap();
    assert!(cases.len() >= 100);
    let mut failed = Vec::new();
    for c in cases {
        let got = run(c["kind"].as_str().unwrap(), &c["input"]);
        if got != c["expected"] {
            failed.push(format!("{}: got {} want {}", c["id"], got, c["expected"]));
        }
    }
    assert!(
        failed.is_empty(),
        "{} of {} failed:\n{}",
        failed.len(),
        cases.len(),
        failed.join("\n")
    );
}
