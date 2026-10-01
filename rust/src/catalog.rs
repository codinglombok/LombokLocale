//! Message catalogs: `locales/<bcp47>/<repo>.json`, one file per language
//! per repo (ARCHITECTURE_UTAMA §7). Catalogs are a flat JSON object of
//! `messageId -> text`. LombokLocale is L0 and cannot depend on
//! LombokJSON (also L0, ADR-016/L0 rule: no mandatory Lombok deps at L0),
//! so this is a small hand-rolled parser scoped to exactly this shape —
//! not a general JSON parser.

use alloc::collections::BTreeMap;
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    UnexpectedEnd,
    UnexpectedChar(char, usize),
    ExpectedObject,
}

/// A loaded message catalog for one locale.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    entries: BTreeMap<String, String>,
}

impl Catalog {
    pub fn get(&self, message_id: &str) -> Option<&str> {
        self.entries.get(message_id).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Iterate `(messageId, text)` pairs in sorted key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn insert(&mut self, id: String, text: String) {
        self.entries.insert(id, text);
    }
}

/// Parse a flat `{"id": "text", ...}` JSON object into a [`Catalog`].
pub fn parse_catalog(json: &str) -> Result<Catalog, CatalogError> {
    let chars: alloc::vec::Vec<char> = json.chars().collect();
    let mut i = 0usize;
    skip_ws(&chars, &mut i);
    expect(&chars, &mut i, '{').map_err(|_| CatalogError::ExpectedObject)?;
    skip_ws(&chars, &mut i);

    let mut catalog = Catalog::default();

    if peek(&chars, i) == Some('}') {
        i += 1;
        skip_ws(&chars, &mut i);
        return match peek(&chars, i) {
            Some(c) => Err(CatalogError::UnexpectedChar(c, i)),
            None => Ok(catalog),
        };
    }

    loop {
        skip_ws(&chars, &mut i);
        let key = parse_string(&chars, &mut i)?;
        skip_ws(&chars, &mut i);
        expect(&chars, &mut i, ':')?;
        skip_ws(&chars, &mut i);
        let value = parse_string(&chars, &mut i)?;
        catalog.insert(key, value);
        skip_ws(&chars, &mut i);
        match peek(&chars, i) {
            Some(',') => {
                i += 1;
            }
            Some('}') => {
                i += 1;
                break;
            }
            Some(c) => return Err(CatalogError::UnexpectedChar(c, i)),
            None => return Err(CatalogError::UnexpectedEnd),
        }
    }

    // Strict JSON: nothing but whitespace may follow the closing brace.
    skip_ws(&chars, &mut i);
    if let Some(c) = peek(&chars, i) {
        return Err(CatalogError::UnexpectedChar(c, i));
    }
    Ok(catalog)
}

fn peek(chars: &[char], i: usize) -> Option<char> {
    chars.get(i).copied()
}

fn skip_ws(chars: &[char], i: &mut usize) {
    while let Some(c) = chars.get(*i) {
        if c.is_whitespace() {
            *i += 1;
        } else {
            break;
        }
    }
}

fn expect(chars: &[char], i: &mut usize, c: char) -> Result<(), CatalogError> {
    match chars.get(*i) {
        Some(x) if *x == c => {
            *i += 1;
            Ok(())
        }
        Some(x) => Err(CatalogError::UnexpectedChar(*x, *i)),
        None => Err(CatalogError::UnexpectedEnd),
    }
}

fn parse_string(chars: &[char], i: &mut usize) -> Result<String, CatalogError> {
    expect(chars, i, '"')?;
    let mut out = String::new();
    loop {
        let c = *chars.get(*i).ok_or(CatalogError::UnexpectedEnd)?;
        *i += 1;
        match c {
            '"' => return Ok(out),
            '\\' => {
                let esc = *chars.get(*i).ok_or(CatalogError::UnexpectedEnd)?;
                *i += 1;
                match esc {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'u' => {
                        let hex: String = (0..4)
                            .map(|_| {
                                let h = *chars.get(*i).unwrap_or(&'0');
                                *i += 1;
                                h
                            })
                            .collect();
                        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                            // JSON requires exactly 4 hex digits; anything else is dropped
                        } else if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(code) {
                                out.push(ch);
                            }
                        }
                    }
                    other => return Err(CatalogError::UnexpectedChar(other, *i)),
                }
            }
            other => out.push(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_catalog() {
        let json = r#"{"greeting": "Halo, {name}!", "farewell": "Sampai jumpa"}"#;
        let cat = parse_catalog(json).unwrap();
        assert_eq!(cat.get("greeting"), Some("Halo, {name}!"));
        assert_eq!(cat.get("farewell"), Some("Sampai jumpa"));
        assert_eq!(cat.len(), 2);
    }

    #[test]
    fn parses_empty_catalog() {
        let cat = parse_catalog("{}").unwrap();
        assert!(cat.is_empty());
    }

    #[test]
    fn handles_escapes() {
        let json = r#"{"quote": "Dia bilang \"halo\"\n"}"#;
        let cat = parse_catalog(json).unwrap();
        assert_eq!(cat.get("quote"), Some("Dia bilang \"halo\"\n"));
    }

    #[test]
    fn unicode_escape_requires_four_hex_digits() {
        let c = parse_catalog(r#"{"a":"\u+041","b":"\u00zz","c":"\u0041","d":"\ud800"}"#).unwrap();
        assert_eq!(c.get("a"), Some(""));
        assert_eq!(c.get("b"), Some(""));
        assert_eq!(c.get("c"), Some("A"));
        assert_eq!(c.get("d"), Some(""));
    }

    #[test]
    fn rejects_trailing_garbage_but_allows_trailing_whitespace() {
        assert!(matches!(
            parse_catalog("{\"a\":\"b\"} x"),
            Err(CatalogError::UnexpectedChar('x', _))
        ));
        assert!(parse_catalog("{\"a\":\"b\"} \n\t").is_ok());
        assert!(matches!(
            parse_catalog("{} {}"),
            Err(CatalogError::UnexpectedChar('{', _))
        ));
    }

    #[test]
    fn rejects_non_object() {
        assert_eq!(parse_catalog("[1,2,3]"), Err(CatalogError::ExpectedObject));
    }

    #[test]
    fn missing_key_returns_none() {
        let cat = parse_catalog(r#"{"a": "b"}"#).unwrap();
        assert_eq!(cat.get("nonexistent"), None);
    }
}
