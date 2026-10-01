//! Cross-language conformance (ADR-002/015). Runs every case in
//! `vectors/lomboklocale-vectors-v1.json` against this implementation.
//! `LOMBOK_REGEN=1 cargo test --test vectors` rewrites the expected `out`/`err`
//! from THIS reference implementation (review the diff before committing).

use lomboklocale::*;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::PathBuf;

fn path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vectors/lomboklocale-vectors-v1.json")
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str()
        .unwrap_or_else(|| panic!("missing string {k} in {v}"))
}

fn f64_of(v: &Value) -> f64 {
    match v {
        Value::String(t) => match t.as_str() {
            "NaN" => f64::NAN,
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            other => other.parse().unwrap(),
        },
        n => n.as_f64().unwrap(),
    }
}

fn i64_of(v: &Value) -> i64 {
    match v {
        Value::String(t) => t.parse().unwrap(),
        n => n.as_i64().unwrap(),
    }
}

fn args_of(v: &Value) -> Vec<(String, ArgValue)> {
    v.as_object()
        .unwrap()
        .iter()
        .map(|(k, val)| {
            let a = match val {
                Value::String(t) => ArgValue::Str(t.clone()),
                n if n.is_u64() => ArgValue::UInt(n.as_u64().unwrap()),
                n if n.is_i64() => ArgValue::Int(n.as_i64().unwrap()),
                n => ArgValue::Float(n.as_f64().unwrap()),
            };
            (k.clone(), a)
        })
        .collect()
}

fn ok(v: Value) -> Value {
    json!({ "out": v })
}
fn err(kind: &str) -> Value {
    json!({ "err": kind })
}

fn eval(case: &Value) -> Value {
    let inp = &case["in"];
    match s(case, "fn") {
        "parse_bcp47" => match parse_bcp47(inp.as_str().unwrap()) {
            Ok(t) => ok(json!(t.to_string())),
            Err(LocaleError::Empty) => err("Empty"),
            Err(LocaleError::InvalidSubtag(_)) => err("InvalidSubtag"),
        },
        "negotiate" => {
            let list = |k: &str| -> Vec<String> {
                inp[k]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect()
            };
            let (r, a) = (list("requested"), list("available"));
            let rr: Vec<&str> = r.iter().map(|x| x.as_str()).collect();
            let aa: Vec<&str> = a.iter().map(|x| x.as_str()).collect();
            ok(json!(negotiate(&rr, &aa, s(inp, "default"))))
        }
        "plural_category" => ok(json!(plural_category(
            s(inp, "locale"),
            inp["n"].as_u64().unwrap()
        )
        .as_str())),
        "ordinal_category" => ok(json!(ordinal_category(
            s(inp, "locale"),
            inp["n"].as_u64().unwrap()
        )
        .as_str())),
        "format_integer" => ok(json!(format_integer(
            i64_of(&inp["value"]),
            s(inp, "locale")
        ))),
        "format_float" => ok(json!(format_float(
            f64_of(&inp["value"]),
            inp["decimals"].as_u64().unwrap() as usize,
            s(inp, "locale")
        ))),
        "format_currency" => ok(json!(format_currency(
            f64_of(&inp["value"]),
            s(inp, "code"),
            s(inp, "locale")
        ))),
        "format_date" => {
            let style = match s(inp, "style") {
                "iso" => DateStyle::Iso,
                "short" => DateStyle::Short,
                _ => DateStyle::Long,
            };
            let d = SimpleDate {
                year: inp["year"].as_i64().unwrap() as i32,
                month: inp["month"].as_u64().unwrap() as u8,
                day: inp["day"].as_u64().unwrap() as u8,
            };
            ok(json!(format_date(d, s(inp, "locale"), style)))
        }
        "parse_iso_date" => match parse_iso_date(inp.as_str().unwrap()) {
            Some(d) => ok(json!({"year": d.year, "month": d.month, "day": d.day})),
            None => ok(Value::Null),
        },
        "format_message" => {
            let a = args_of(&inp["args"]);
            let refs: Vec<(&str, ArgValue)> =
                a.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
            match format_message(s(inp, "locale"), s(inp, "pattern"), &refs) {
                Ok(t) => ok(json!(t)),
                Err(MessageError::UnbalancedBraces) => err("UnbalancedBraces"),
                Err(MessageError::UnknownVariable(_)) => err("UnknownVariable"),
                Err(MessageError::MalformedSelector(_)) => err("MalformedSelector"),
                Err(MessageError::TooDeep) => err("TooDeep"),
            }
        }
        "parse_catalog" => match parse_catalog(inp.as_str().unwrap()) {
            Ok(c) => {
                let mut m = Map::new();
                for (k, v) in c.iter() {
                    m.insert(k.to_string(), json!(v));
                }
                ok(Value::Object(m))
            }
            Err(CatalogError::ExpectedObject) => err("ExpectedObject"),
            Err(CatalogError::UnexpectedEnd) => err("UnexpectedEnd"),
            Err(CatalogError::UnexpectedChar(..)) => err("UnexpectedChar"),
        },
        "resolve" => {
            let mut cat = Catalog::default();
            for (k, v) in inp["catalog"].as_object().unwrap() {
                cat.insert(k.clone(), v.as_str().unwrap().to_string());
            }
            let a = args_of(&inp["args"]);
            let refs: Vec<(&str, ArgValue)> =
                a.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
            let m = resolve(
                s(inp, "locale"),
                &cat,
                s(inp, "code"),
                s(inp, "messageId"),
                &refs,
            );
            ok(json!({"code": m.code, "messageId": m.message_id, "text": m.text}))
        }
        other => panic!("unknown fn {other}"),
    }
}

#[test]
fn conformance() {
    let raw = fs::read_to_string(path()).unwrap();
    let doc: Value = serde_json::from_str(&raw).unwrap();
    let cases = doc["cases"].as_array().unwrap();
    let regen = std::env::var("LOMBOK_REGEN").is_ok();
    let mut out_lines = Vec::new();
    let mut failures = Vec::new();
    for (i, c) in cases.iter().enumerate() {
        let got = eval(c);
        let mut merged = json!({"fn": c["fn"], "in": c["in"]});
        for (k, v) in got.as_object().unwrap() {
            merged[k] = v.clone();
        }
        if !regen {
            let expected_out = c.get("out");
            let expected_err = c.get("err");
            if expected_out != got.get("out") || expected_err != got.get("err") {
                failures.push(format!(
                    "case {i} {}: in={} expected={:?}/{:?} got={}",
                    c["fn"], c["in"], expected_out, expected_err, got
                ));
            }
        }
        out_lines.push(serde_json::to_string(&merged).unwrap());
    }
    if regen {
        let body = format!(
            "{{\"suite\":\"lomboklocale\",\"version\":1,\"cases\":[\n{}\n]}}\n",
            out_lines.join(",\n")
        );
        fs::write(path(), body).unwrap();
        return;
    }
    assert!(
        failures.is_empty(),
        "{} vector failures, first 5:\n{}",
        failures.len(),
        failures
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
