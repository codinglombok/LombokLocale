# LombokLocale — API v0.2.0

Public surface of the `lomboklocale` 0.2.0 packages, port by port. Every
function listed here is part of the cross-language contract: the same input
produces byte-identical output in every port, and every failure raises an
error carrying one of the 16 codes below (see
[SPEC_LombokLocale_v0.2.0.md](SPEC_LombokLocale_v0.2.0.md), sections 3-8).
The naming style follows each language's convention (snake_case, camelCase,
PascalCase); the mapping is one-to-one.

## Error model

All ports use the same 16 error codes. The vectors compare codes, not
human-readable text, so port-specific message wording is free.

| Code | Raised by |
|---|---|
| `EMPTY` | `parseTag("")` |
| `INVALID_SUBTAG` | malformed, out-of-order or unsupported BCP 47 pieces |
| `DUPLICATE_VARIANT` / `DUPLICATE_EXTENSION` | repeated variant or extension singleton |
| `BAD_NUMBER` | unparseable decimal text or out-of-range exponent (section 3 of the SPEC) |
| `BAD_OPTION` | unknown style, bad currency code, or fraction bounds outside 0..20 |
| `BAD_DATE` | date text outside `YYYY-MM-DD` / 0001-01-01..9999-12-31 |
| `SYNTAX` | malformed MessageFormat pattern |
| `MISSING_OTHER` | plural/select/ordinal choice without an `other` key |
| `MISSING_ARGUMENT` | placeholder name absent from the arguments |
| `BAD_ARGUMENT` | argument value of the wrong kind for its placeholder |
| `TOO_DEEP` | MessageFormat nesting deeper than `MAX_DEPTH` (16) |
| `BAD_JSON` / `NOT_OBJECT` / `NON_STRING_VALUE` / `DUPLICATE_KEY` | catalog parsing (section 8 of the SPEC) |

## Rust (`lomboklocale` crate, `rust/`)

```rust
use lomboklocale::{parse_tag, canonicalize, negotiate, data_locale, LanguageTag,
    plural_category, ordinal_category, PluralCategory,
    format_number, NumberOptions, Style, format_date, Date, DateStyle,
    format_message, Arg, Args, MAX_DEPTH, parse_catalog, Catalog, resolve,
    LocalizedMessage, CLDR_VERSION, MAX_EXPONENT};
```

| Export | Signature | Notes |
|---|---|---|
| `parse_tag` | `fn(&str) -> Result<LanguageTag>` | fields `language`, `script`, `region`, `variants`, `extensions`, `private_use`; `base()` and `lookup_chain()` helpers |
| `canonicalize` | `fn(&str) -> Result<String>` | lowercase/case-fixed canonical form |
| `negotiate` | `fn(requested: &[impl AsRef<str>], available: &[impl AsRef<str>], default: &str) -> String` | RFC 4647 lookup |
| `data_locale` | `fn(&str) -> &'static str` | one of the 28 data locales, `en` fallback |
| `plural_category` / `ordinal_category` | `fn(&str, &str) -> Result<PluralCategory>` | `PluralCategory` = zero/one/two/few/many/other |
| `format_number` | `fn(locale: &str, value: &str, &NumberOptions) -> Result<String>` | `NumberOptions::{decimal, percent, currency(code), fraction(min, max)}`; `Style` = Decimal/Percent/Currency |
| `format_date` | `fn(locale: &str, date: &str, DateStyle) -> Result<String>` | `Date::parse`, `Date::weekday`; `DateStyle` = Full/Long/Medium/Short |
| `format_message` | `fn(locale: &str, pattern: &str, args: Args) -> Result<String>` | `Args<'a> = &[(&str, Arg)]`; `Arg::from(...)`, `Arg::str`, `Arg::num` |
| `parse_catalog` | `fn(json: &str) -> Result<Catalog>` | `Catalog::{get, len, is_empty, iter, insert}` |
| `resolve` | `fn(locale, catalog: &Catalog, code: &str, message_id: &str, args: Args) -> LocalizedMessage` | never fails; sets `fallback` and `text = "!!id!!"` when the message is missing |
| consts | `CLDR_VERSION = "47.0.0"`, `MAX_EXPONENT = 1000`, `MAX_DEPTH = 16` | `Error { code: ErrorCode, detail: String }`, `Display` as `CODE: detail` |

```rust
let s = format_number("id", "15000", &NumberOptions::currency("IDR"))?;   // "Rp 15.000,00"
let m = resolve("id", &parse_catalog(r#"{"hi": "Halo {name}"}"#)?, "GREET", "hi",
                &[("name", Arg::from("dunia"))]);
assert_eq!(m.text, "Halo dunia");
```

## TypeScript / JavaScript (`lomboklocale` on npm, `typescript/`)

Dual ESM and CJS builds: `import { ... } from 'lomboklocale'` and
`require('lomboklocale')` both resolve (ESM: `dist/index.js`, CJS:
`dist/cjs/index.js` with a `commonjs` marker).

| Export | Signature | Notes |
|---|---|---|
| `parseTag(tag: string): LanguageTag` | fields `language`, `script`, `region`, `variants`, `extensions`, `privateUse`; `baseTag(t)`, `formatTag(t)`, `lookupChain(t)` |
| `canonicalize(tag: string): string` | throws `LocaleError` on bad input |
| `negotiate(requested, available: readonly string[], defaultLocale: string): string` | RFC 4647 lookup |
| `dataLocale(tag: string): string` | 28 data locales, `en` fallback |
| `pluralCategory(locale, value: Numeric): PluralCategory` / `ordinalCategory` | `Numeric = number \| bigint \| string`; `PluralCategory` union |
| `formatNumber(locale: string, value: Numeric, opts?: NumberOptions): string` | `NumberOptions { style?, currency?, minFraction?, maxFraction? }` |
| `parseDate(s): {year, month, day}` / `weekday(y, m, d): number` / `formatDate(locale, date, style?: DateStyle): string` | `DateStyle = 'full' \| 'long' \| 'medium' \| 'short'` |
| `formatMessage(locale, pattern: string, args?: Args): string` | `Arg = string \| number \| bigint \| { num: string }`; `Args = Record<string, Arg>` |
| `parseCatalog(json: string): Catalog` | `Catalog = Map<string, string>` |
| `resolve(locale, catalog, code, messageId, args?): LocalizedMessage` | `{ code, messageId, text, fallback }` |
| `LocaleError` | `class LocaleError extends Error` with `code: ErrorCode` |
| consts | `CLDR_VERSION`, `MAX_DEPTH`, `MAX_EXPONENT`, `toDecimalString(v: Numeric): string` |

```ts
import { formatNumber, pluralCategory, formatMessage } from 'lomboklocale';
formatNumber('de', '15000', { style: 'currency', currency: 'EUR' }); // "15.000,00 €"
pluralCategory('pl', 21);                                           // "many"
formatMessage('en', '{n, plural, one{# file} other{# files}}', { n: 1234 }); // "1,234 files"
```

## Python (`lomboklocale` on PyPI, `python/`)

One flat module, typed (`py.typed`), zero dependencies, Python 3.9+.

| Export | Signature |
|---|---|
| `parse_tag(tag: str) -> LanguageTag` | dataclass; `.base()`, `.lookup_chain()` |
| `canonicalize(tag: str) -> str` |
| `negotiate(requested, available, default: str) -> str` |
| `data_locale(tag: str) -> str` |
| `plural_category(locale: str, value: Numeric) -> str` / `ordinal_category` | `Numeric = int \| float \| Decimal \| str`; exact values: pass `str` or `Decimal`
| `format_number(locale, value: Numeric, style: str = 'decimal', currency=None, min_fraction=None, max_fraction=None) -> str` |
| `parse_date(s: str) -> tuple[int, int, int]` / `weekday(y, m, d) -> int` / `format_date(locale, date, style: str = 'medium') -> str` |
| `format_message(locale, pattern: str, args: Mapping[str, Arg] \| None = None) -> str` | text args are `str`; number args may be `int`, `float`, `Decimal` or `Num("...")` |
| `parse_catalog(text: str) -> dict[str, str]` |
| `resolve(locale, catalog, code: str, message_id: str, args=None) -> LocalizedMessage` | dataclass `{code, message_id, text, fallback}` |
| `LocaleError`, `Num`, `to_decimal_string(v) -> str`, `CLDR_VERSION`, `MAX_DEPTH`, `MAX_EXPONENT` |

```python
from lomboklocale import format_number, format_date, Num
format_number("fr", 1234.5, style="currency", currency="EUR")  # "1 234,50 €"
format_date("ja", "2026-10-02", "full")                         # "2026年10月2日金曜日"
format_number("en", Num("0.1") + Num("0.2"), min_fraction=2)   # "0.30"
```

## Go (`github.com/codinglombok/lomboklocale/go`, tag `go/v0.2.0`)

Package `lomboklocale`, module path `github.com/codinglombok/lomboklocale/go`,
Go 1.22+. Table-driven tests mirror the vectors; no dependencies.

| Export | Signature |
|---|---|
| `CLDRVersion() string` |
| `ParseTag(tag string) (LanguageTag, error)` |
| `Canonicalize(tag string) (string, error)` |
| `Negotiate(requested, available []string, def string) string` |
| `DataLocale(tag string) string` |
| `FloatString(f float64) string` | shortest exact decimal text of a float |
| `PluralCategory(locale, value string) (string, error)` / `OrdinalCategory` |
| `FormatNumber(locale, value string, opts NumberOptions) (string, error)` | `NumberOptions{Style, Currency, MinFraction, MaxFraction}`; `Digits(n int) *int` helper |
| `ParseDate(s string) (year, month, day int, err error)` / `Weekday(y, m, d int) int` |
| `FormatDate(locale, date, style string) (string, error)` |
| `Str(s string) Arg` / `Num(s string) Arg` / `Int(v int64) Arg` / `Float(v float64) Arg` | `Arg` is the message argument value |
| `FormatMessage(locale, pattern string, args map[string]Arg) (string, error)` |
| `ParseCatalog(text string) (map[string]string, error)` |
| `Resolve(locale string, catalog map[string]string, code, messageID string, args map[string]Arg) LocalizedMessage` |

```go
s, err := lomboklocale.FormatNumber("en", "0.256", lomboklocale.NumberOptions{Style: "percent"}) // "26%"
tag, err := lomboklocale.ParseTag("id-ID-u-ca-islamic")
```

## PHP (`codinglombok/lomboklocale` on Packagist, `php/`)

Namespace `LombokLocale`, PHP 8.1+. The autoloader maps `LombokLocale\` to
`php/src/` (root `composer.json`); the tests run without Composer via
`php/tests/bootstrap.php`.

| Export | Signature |
|---|---|
| `Locale::cldrVersion(): string` |
| `Locale::parseTag(string $tag): LanguageTag` | `LanguageTag` with `->__toString()`; throws `LocaleError` |
| `Locale::canonicalize(string $tag): string` |
| `Locale::negotiate(array $requested, array $available, string $default): string` |
| `Locale::dataLocale(string $tag): string` |
| `Locale::toDecimalString(int\|float\|string\|Num $v): string` |
| `Locale::pluralCategory(string $locale, int\|float\|string\|Num $value): string` / `ordinalCategory` |
| `Locale::formatNumber(string $locale, int\|float\|string\|Num $value, string $style = 'decimal', ?string $currency = null, ?int $minFraction = null, ?int $maxFraction = null): string` |
| `Locale::parseDate(string $s): array` / `Locale::weekday(int $y, int $m, int $d): int` |
| `Locale::formatDate(string $locale, string $date, string $style = 'medium'): string` |
| `Locale::formatMessage(string $locale, string $pattern, array $args = []): string` | `Num` instances pass exact decimals |
| `Locale::parseCatalog(string $json): array` |
| `Locale::resolve(string $locale, array $catalog, string $code, string $messageId, array $args = []): LocalizedMessage` | readonly `code`, `messageId`, `text`, `fallback` |
| `LocaleError` | `final class LocaleError extends InvalidArgumentException`, `->errorCode` |
| `Num` | exact decimal wrapper, `new Num('0.1')` |

```php
use LombokLocale\Locale;
use LombokLocale\Num;

Locale::formatNumber('ko', 15000, 'currency', 'KRW');          // "₩15,000"
Locale::formatNumber('en', new Num('0.1'), 'decimal', null, 2); // "0.10"
```

## Where the behavior is pinned

- `vectors/lomboklocale-vectors-v1.json` — 342 cases; each port runs all of them in CI
  (`rust/tests/vectors.rs`, `typescript/tests/vectors.test.ts`,
  `python/tests/test_vectors.py`, `go/vectors_test.go`, `php/tests/vectors.php`).
- `rust/tests/api.rs` and the per-port `api` test files cover the error codes.
- `rust/tests/catalogs.rs` validates every `locales/*/lomboklocale.json` catalog.

*Document license: Apache-2.0 OR MIT. (c) codinglombok*
