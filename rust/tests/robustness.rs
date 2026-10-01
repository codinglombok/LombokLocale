//! Dependency-free pseudo-fuzz over every public entry point.
use lomboklocale::*;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str {
        xs[(self.next() % xs.len() as u64) as usize]
    }
}

const PIECES: &[&str] = &[
    "{",
    "}",
    ",",
    " ",
    "plural",
    "select",
    "selectordinal",
    "one",
    "other",
    "few",
    "*",
    "#",
    "n",
    "name",
    "x",
    "\\",
    "\"",
    "en",
    "id",
    "-",
    "_",
    "US",
    "Hans",
    "zh",
    "ar",
    "\u{0}",
    "é",
    "日",
    "1",
    "9999999999999999999",
    ":",
];

fn soup(rng: &mut Rng, max: u64) -> String {
    (0..rng.next() % max).map(|_| rng.pick(PIECES)).collect()
}

#[test]
fn nothing_panics() {
    let mut rng = Rng(0x2545F4914F6CDD1D);
    let args = [
        ("n", ArgValue::UInt(3)),
        ("name", ArgValue::Str("z".into())),
        ("x", ArgValue::Float(1.5)),
    ];
    for _ in 0..40_000 {
        let s = soup(&mut rng, 30);
        let t = soup(&mut rng, 12);
        let _ = parse_bcp47(&s);
        let _ = negotiate(&[&s, &t], &["en", "id", "zh-Hans"], "en");
        let _ = parse_catalog(&s);
        let _ = format_message(&t, &s, &args);
        let n = rng.next();
        let _ = plural_category(&t, n);
        let _ = ordinal_category(&t, n);
        let _ = format_integer(n as i64, &t);
        let _ = format_float(f64::from_bits(n), (n % 50) as usize, &t);
        let _ = format_currency(f64::from_bits(rng.next()), &s, &t);
        let _ = parse_iso_date(&s);
        let _ = format_date(
            SimpleDate {
                year: n as i32,
                month: (n >> 8) as u8,
                day: (n >> 16) as u8,
            },
            &t,
            DateStyle::Long,
        );
    }
}
