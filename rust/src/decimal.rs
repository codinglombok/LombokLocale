//! Exact decimal values given as text (SPEC section 3).

use crate::error::{Error, ErrorCode, Result};
use alloc::string::{String, ToString};

/// A parsed number: finite with exact digits, or NaN/Infinity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Dec {
    Finite {
        neg: bool,
        int: String,
        frac: String,
    },
    NaN,
    Inf {
        neg: bool,
    },
}

/// Largest accepted exponent magnitude (SPEC section 3).
pub const MAX_EXPONENT: i64 = 1000;

/// Parses `[+-]digits[.digits][(e|E)[+-]digits]`, `NaN`, `Infinity`,
/// `+Infinity` or `-Infinity`.
pub(crate) fn parse(s: &str) -> Result<Dec> {
    match s {
        "NaN" => return Ok(Dec::NaN),
        "Infinity" | "+Infinity" => return Ok(Dec::Inf { neg: false }),
        "-Infinity" => return Ok(Dec::Inf { neg: true }),
        _ => {}
    }
    let bad = || Error::new(ErrorCode::BadNumber, s);
    let b = s.as_bytes();
    let mut i = 0;
    let mut neg = false;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        neg = b[i] == b'-';
        i += 1;
    }
    let start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return Err(bad());
    }
    let mut digits = String::from(&s[start..i]);
    let mut point = digits.len() as i64;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == fs {
            return Err(bad());
        }
        digits.push_str(&s[fs..i]);
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        let mut eneg = false;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            eneg = b[i] == b'-';
            i += 1;
        }
        let es = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == es || i - es > 4 {
            return Err(bad());
        }
        let e: i64 = s[es..i].parse().map_err(|_| bad())?;
        if e > MAX_EXPONENT {
            return Err(bad());
        }
        point += if eneg { -e } else { e };
    }
    if i != b.len() {
        return Err(bad());
    }
    if point <= 0 {
        let mut z = String::new();
        for _ in 0..(1 - point) {
            z.push('0');
        }
        z.push_str(&digits);
        digits = z;
        point = 1;
    }
    while (digits.len() as i64) < point {
        digits.push('0');
    }
    let (ip, fp) = digits.split_at(point as usize);
    let ip = ip.trim_start_matches('0');
    Ok(Dec::Finite {
        neg,
        int: if ip.is_empty() {
            "0".to_string()
        } else {
            ip.to_string()
        },
        frac: fp.to_string(),
    })
}

/// Rounds `frac` to at most `max` digits, half away from zero.
pub(crate) fn round(int: &str, frac: &str, max: usize) -> (String, String) {
    if frac.len() <= max {
        return (int.to_string(), frac.to_string());
    }
    let mut all: alloc::vec::Vec<u8> = int.bytes().chain(frac[..max].bytes()).collect();
    if frac.as_bytes()[max] >= b'5' {
        let mut k = all.len();
        loop {
            if k == 0 {
                all.insert(0, b'1');
                break;
            }
            k -= 1;
            if all[k] == b'9' {
                all[k] = b'0';
            } else {
                all[k] += 1;
                break;
            }
        }
    }
    let split = all.len() - max;
    let ip = String::from_utf8(all[..split].to_vec()).expect("ascii");
    let fp = String::from_utf8(all[split..].to_vec()).expect("ascii");
    let ip = ip.trim_start_matches('0');
    (
        if ip.is_empty() {
            "0".to_string()
        } else {
            ip.to_string()
        },
        fp,
    )
}

/// Remainder of a decimal digit string divided by `m` (m > 0).
pub(crate) fn digits_mod(digits: &str, m: u64) -> u64 {
    digits
        .bytes()
        .fold(0u64, |r, c| (r * 10 + u64::from(c - b'0')) % m)
}

/// Value of a digit string, or None above 10^18.
pub(crate) fn digits_value(digits: &str) -> Option<u64> {
    if digits.len() > 18 {
        None
    } else {
        digits.parse().ok()
    }
}
