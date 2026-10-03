//! Types of the generated CLDR tables in `data.rs`.

/// One plural relation: `operand [% modulus] (= | !=) ranges`.
#[derive(Debug)]
pub struct Rel {
    /// Operand letter: `n`, `i`, `v`, `w`, `f`, `t`, `c` or `e`.
    pub op: u8,
    /// Modulus, 0 when absent.
    pub modulus: u32,
    /// True for `!=`.
    pub negated: bool,
    /// Inclusive ranges.
    pub ranges: &'static [(u32, u32)],
}

/// Plural rules by locale: (category index 0..=4, OR groups of AND relations).
pub type RuleTable = &'static [(&'static str, &'static [(u8, &'static [&'static [Rel]])])];

/// Number and date data of one locale (CLDR 47, Latin digits, Gregorian calendar).
#[derive(Debug)]
pub struct LocaleData {
    pub decimal: &'static str,
    pub group: &'static str,
    pub minus: &'static str,
    pub percent: &'static str,
    pub infinity: &'static str,
    pub nan: &'static str,
    pub decimal_pattern: &'static str,
    pub percent_pattern: &'static str,
    pub currency_pattern: &'static str,
    pub era: &'static str,
    pub min_grouping: usize,
    /// (ISO code, symbol, first char is S/Z, last char is S/Z).
    pub currency_symbols: &'static [(&'static str, &'static str, bool, bool)],
    /// full, long, medium, short.
    pub date_formats: [&'static str; 4],
    /// abbreviated, wide.
    pub months: [[&'static str; 12]; 2],
    /// abbreviated, wide; Sunday first.
    pub days: [[&'static str; 7]; 2],
}
