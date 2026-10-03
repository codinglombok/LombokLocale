# LombokLocale

> BCP 47 tags and negotiation, CLDR 47 plural rules, number/percent/currency and date formatting, an ICU MessageFormat subset, and JSON message catalogs. The same input gives the same text in Rust, TypeScript, Python, Go and PHP. No runtime dependencies.

[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](LICENSE-APACHE)
[![CI](https://github.com/codinglombok/LombokLocale/actions/workflows/ci.yml/badge.svg)](https://github.com/codinglombok/LombokLocale/actions/workflows/ci.yml)
[![Vectors](https://img.shields.io/badge/shared%20vectors-342%20x%205%20ports-success)](vectors/)
[![CLDR](https://img.shields.io/badge/CLDR-47.0-informational)](data/)
[![Lombok Ecosystem](https://img.shields.io/badge/Lombok-Ecosystem-2e7d5b?logo=github)](https://github.com/codinglombok)

Part of the [Lombok Ecosystem](https://github.com/codinglombok).

## Mengapa library ini? (Why this library?)

- **The same text everywhere.** `Intl` in browsers, ICU in PHP, Babel in Python and hand-written code in Go each ship a different CLDR version and different defaults, so an invoice total or a "3 files" message can differ between your frontend and backend. LombokLocale pins CLDR 47 and runs 342 shared cases ([SPEC](docs/SPEC_LombokLocale_v0.2.0.md)) in CI for every port.
- **Checked against ICU.** Number, currency, percent, date and plural results are cross-checked against ICU 77.1; the four known differences are listed in the SPEC.
- **Exact decimals.** Numbers are taken as decimal text, so `0.1 + 0.2`, `1e21` or a 30-digit amount are rounded exactly (half away from zero), never through binary floating point.
- **Small and embeddable.** No dependencies; the Rust crate is `no_std` + `alloc`; data for 28 locales and plural rules for all CLDR locales are embedded.

## Installation

| Language | Package | Status |
|---|---|---|
| Rust | `lomboklocale` (crates.io) | not yet published |
| TypeScript / JavaScript | `lomboklocale` (npm) | not yet published |
| Python | `lomboklocale` (PyPI) | not yet published |
| Go | `github.com/codinglombok/lomboklocale/go` | tag `go/v0.2.0` on release |
| PHP | `codinglombok/lomboklocale` (Packagist) | needs a split repository first |

## Quick start

```ts
import { formatNumber, formatDate, formatMessage, negotiate, pluralCategory } from 'lomboklocale';

negotiate(['id-ID', 'en'], ['en', 'id'], 'en');                         // "id"
formatNumber('id', 1234567.891);                                        // "1.234.567,891"
formatNumber('de', '15000', { style: 'currency', currency: 'EUR' });    // "15.000,00 €"
formatNumber('hi', '1234567.5');                                        // "12,34,567.5"
formatDate('id', '2026-10-02', 'full');                                 // "Jumat, 02 Oktober 2026"
pluralCategory('pl', 21);                                               // "many"
formatMessage('en', '{n, plural, one{# file} other{# files}}', { n: 1234 });   // "1,234 files"
formatMessage('en', '{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}', { n: 23 }); // "23rd"
```

```rust
use lomboklocale::{format_message, format_number, Arg, NumberOptions};

format_number("id", "15000", &NumberOptions::currency("IDR"))?;     // "Rp 15.000,00"
format_message("id", "Total {n}", &[("n", Arg::from(1234567.5))])?; // "Total 1.234.567,5"
```

```python
from lomboklocale import format_date, format_number, Num

format_number("fr", 1234.5, style="currency", currency="EUR")   # "1 234,50 €"
format_date("ja", "2026-10-02", "full")                          # "2026年10月2日金曜日"
```

```go
s, err := lomboklocale.FormatNumber("en", "0.256", lomboklocale.NumberOptions{Style: "percent"}) // "26%"
```

```php
use LombokLocale\Locale;

Locale::formatNumber('ko', 15000, 'currency', 'KRW');   // "₩15,000"
```

Spaces shown above inside amounts are U+00A0 or U+202F, as CLDR specifies.

## Message catalogs

`locales/<tag>/lomboklocale.json` holds the messages of this library itself; `en` and `id` are reviewed, the others are machine-translated drafts awaiting native review (see `locales/STATUS.md`). Any application can use the same format: `parse_catalog` reads a strict JSON object of strings, and `resolve` formats one message or returns `!!id!!` so a missing translation is visible.

## Known limitations

Latin digits and the Gregorian calendar only; number/date data for 28 locales (other locales fall back to `en` for formatting but use their own plural rules); no time-of-day, time zones, relative time, lists, units, compact or scientific notation; MessageFormat is a subset (no `choice`, `spellout`, `ordinal` styles, no MessageFormat 2 syntax). See [docs/full_summary_project_LombokLocale_v0.2.0.md](docs/full_summary_project_LombokLocale_v0.2.0.md).

## Development

```bash
python3 scripts/gen_ports.py --check && python3 vectors/build_vectors.py && node vectors/check_icu.mjs
cd rust && cargo test && cargo clippy --all-targets -- -D warnings
cd typescript && npm ci && npm run coverage
cd python && python -m pytest
cd go && go test ./...
cd php && php tests/run.php
bash scripts/lombok-doctor.sh LombokLocale
```

## License

Code: Apache-2.0 OR MIT, at your option ([LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT)). Embedded CLDR data: Unicode License v3 ([LICENSE-UNICODE](LICENSE-UNICODE)).
