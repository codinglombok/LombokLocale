//! Message catalogs: strict JSON objects of strings (SPEC section 8).

use crate::error::{Error, ErrorCode, Result};
use alloc::collections::BTreeMap;
use alloc::string::String;

/// Message id to pattern.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    entries: BTreeMap<String, String>,
}

impl Catalog {
    /// The pattern of `message_id`.
    pub fn get(&self, message_id: &str) -> Option<&str> {
        self.entries.get(message_id).map(String::as_str)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Adds or replaces an entry.
    pub fn insert(&mut self, id: impl Into<String>, pattern: impl Into<String>) {
        self.entries.insert(id.into(), pattern.into());
    }
}

struct P<'a> {
    b: &'a [u8],
    i: usize,
}

const MAX_DEPTH: usize = 64;

impl P<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn bad(&self) -> Error {
        Error::new(ErrorCode::BadJson, alloc::format!("{}", self.i))
    }

    fn string(&mut self) -> Result<String> {
        if self.b.get(self.i) != Some(&b'"') {
            return Err(self.bad());
        }
        self.i += 1;
        let mut out: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
        loop {
            let c = *self.b.get(self.i).ok_or_else(|| self.bad())?;
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = *self.b.get(self.i).ok_or_else(|| self.bad())?;
                    self.i += 1;
                    match e {
                        b'"' | b'\\' | b'/' => out.push(e),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let mut cp = self.hex4()?;
                            if (0xD800..0xDC00).contains(&cp) {
                                if self.b.get(self.i) == Some(&b'\\')
                                    && self.b.get(self.i + 1) == Some(&b'u')
                                {
                                    self.i += 2;
                                    let lo = self.hex4()?;
                                    if !(0xDC00..0xE000).contains(&lo) {
                                        return Err(self.bad());
                                    }
                                    cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                } else {
                                    return Err(self.bad());
                                }
                            } else if (0xDC00..0xE000).contains(&cp) {
                                return Err(self.bad());
                            }
                            let ch = char::from_u32(cp).ok_or_else(|| self.bad())?;
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        _ => return Err(self.bad()),
                    }
                }
                0..=0x1f => return Err(self.bad()),
                _ => out.push(c),
            }
        }
        String::from_utf8(out).map_err(|_| self.bad())
    }

    fn hex4(&mut self) -> Result<u32> {
        let s = self.b.get(self.i..self.i + 4).ok_or_else(|| self.bad())?;
        let mut v = 0u32;
        for &c in s {
            v = v * 16 + (c as char).to_digit(16).ok_or_else(|| self.bad())?;
        }
        self.i += 4;
        Ok(v)
    }

    /// Skips any JSON value (used to report NON_STRING_VALUE only for valid JSON).
    fn skip_value(&mut self, depth: usize) -> Result<()> {
        if depth > MAX_DEPTH {
            return Err(self.bad());
        }
        self.ws();
        match self.b.get(self.i) {
            Some(b'"') => self.string().map(|_| ()),
            Some(b'{') | Some(b'[') => {
                let close = if self.b[self.i] == b'{' { b'}' } else { b']' };
                let obj = close == b'}';
                self.i += 1;
                self.ws();
                if self.b.get(self.i) == Some(&close) {
                    self.i += 1;
                    return Ok(());
                }
                loop {
                    self.ws();
                    if obj {
                        self.string()?;
                        self.ws();
                        if self.b.get(self.i) != Some(&b':') {
                            return Err(self.bad());
                        }
                        self.i += 1;
                    }
                    self.skip_value(depth + 1)?;
                    self.ws();
                    match self.b.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(c) if *c == close => {
                            self.i += 1;
                            return Ok(());
                        }
                        _ => return Err(self.bad()),
                    }
                }
            }
            Some(_) => {
                for lit in [&b"true"[..], b"false", b"null"] {
                    if self.b[self.i..].starts_with(lit) {
                        self.i += lit.len();
                        return Ok(());
                    }
                }
                self.number()
            }
            None => Err(self.bad()),
        }
    }

    fn number(&mut self) -> Result<()> {
        let start = self.i;
        let b = self.b;
        let digits = |i: &mut usize| {
            let s = *i;
            while *i < b.len() && b[*i].is_ascii_digit() {
                *i += 1;
            }
            *i - s
        };
        let mut i = self.i;
        if b.get(i) == Some(&b'-') {
            i += 1;
        }
        let n = digits(&mut i);
        if n == 0 || (n > 1 && b[i - n] == b'0') {
            return Err(self.bad());
        }
        if b.get(i) == Some(&b'.') {
            i += 1;
            if digits(&mut i) == 0 {
                return Err(self.bad());
            }
        }
        if matches!(b.get(i), Some(b'e' | b'E')) {
            i += 1;
            if matches!(b.get(i), Some(b'+' | b'-')) {
                i += 1;
            }
            if digits(&mut i) == 0 {
                return Err(self.bad());
            }
        }
        if i == start {
            return Err(self.bad());
        }
        self.i = i;
        Ok(())
    }
}

/// Parses a catalog: a JSON object (RFC 8259) whose values are all strings.
///
/// # Errors
/// `BAD_JSON` for invalid JSON, `NOT_OBJECT` when the top level is not an
/// object, `NON_STRING_VALUE`, `DUPLICATE_KEY`. Invalid JSON is reported
/// before the other errors.
pub fn parse_catalog(json: &str) -> Result<Catalog> {
    // First pass: validity of the whole document.
    let mut p = P {
        b: json.as_bytes(),
        i: 0,
    };
    p.skip_value(0)?;
    p.ws();
    if p.i != p.b.len() {
        return Err(p.bad());
    }
    let mut p = P {
        b: json.as_bytes(),
        i: 0,
    };
    p.ws();
    if p.b.get(p.i) != Some(&b'{') {
        return Err(Error::new(ErrorCode::NotObject, ""));
    }
    p.i += 1;
    let mut cat = Catalog::default();
    p.ws();
    if p.b.get(p.i) == Some(&b'}') {
        return Ok(cat);
    }
    loop {
        p.ws();
        let key = p.string()?;
        p.ws();
        p.i += 1; // ':' (validated in the first pass)
        p.ws();
        if p.b.get(p.i) != Some(&b'"') {
            return Err(Error::new(ErrorCode::NonStringValue, key));
        }
        let value = p.string()?;
        if cat.entries.contains_key(&key) {
            return Err(Error::new(ErrorCode::DuplicateKey, key));
        }
        cat.entries.insert(key, value);
        p.ws();
        if p.b.get(p.i) == Some(&b',') {
            p.i += 1;
        } else {
            return Ok(cat);
        }
    }
}
