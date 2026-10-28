# LombokLocale — how to use v0.2.0

Installation per registry, a five-minute quickstart, a cookbook of common
tasks, and troubleshooting. Every snippet below is byte-verified by the
repository's own test suites; copy them freely.

## Installation

| Language | Install | Import |
|---|---|---|
| Rust | `cargo add lomboklocale` (crates.io) | `use lomboklocale::...;` |
| TypeScript / JavaScript | `npm install lomboklocale` | `import ... from 'lomboklocale'` or `require('lomboklocale')` |
| Python | `pip install lomboklocale` (PyPI) | `from lomboklocale import ...` |
| Go | `go get github.com/codinglombok/lomboklocale/go@go/v0.2.0` | `import "github.com/codinglombok/lomboklocale/go"` (package `lomboklocale`) |
| PHP | `composer require codinglombok/lomboklocale` | `use LombokLocale\Locale;` |

No port has runtime dependencies, so none of the above pulls anything else.

## Five-minute quickstart

Pick the locale once, format everywhere, and keep numbers exact by passing
them as decimal text. TypeScript example (identical logic in every port):

```ts
import { negotiate, formatNumber, formatDate, formatMessage, pluralCategory } from 'lomboklocale';

// 1. Which locale should we serve? (RFC 4647 lookup over what you support)
const locale = negotiate(['id-ID', 'en-US'], ['en', 'id', 'ja'], 'en');   // "id"

// 2. Money and big numbers: decimal text keeps 0.1+0.2 and 1e21 exact.
formatNumber(locale, '1234567.891');                                    // "1.234.567,891"
formatNumber('de', '15000', { style: 'currency', currency: 'EUR' });     // "15.000,00 €" (nbsp before €)

// 3. Dates from a plain YYYY-MM-DD string.
formatDate('id', '2026-10-02', 'full');                                 // "Jumat, 02 Oktober 2026"

// 4. Plural-aware messages with the ICU subset.
const files = 1234;
pluralCategory('pl', files);                                            // "many" (CLDR 47)
formatMessage('en', '{n, plural, one{# file} other{# files}}', { n: files }); // "1,234 files"
```

Rust equivalent:

```rust
use lomboklocale::{negotiate, format_number, format_message, Arg, NumberOptions};

let locale = negotiate(&["id-ID"], &["en", "id"], "en");
let total  = format_number("id", "15000", &NumberOptions::currency("IDR"))?;
let text   = format_message("en", "Total: {n}", &[("n", Arg::from(1234567.5))])?;
```

## Cookbook

### Exact money math (never through binary floats)

```python
from decimal import Decimal
from lomboklocale import format_number

total = format_number("id", Decimal("45000.30"), style="currency", currency="IDR")
# "Rp 45.000,30"
```

Pass floats only for display; the ports convert them to the shortest
round-tripping decimal text first, matching every other port. For message
arguments, `Num` carries exact decimals: `format_message("en", "Total {n}",
{"n": Num("45000.30")})`.

### Locale-aware strings without a framework

```php
use LombokLocale\Locale;

$catalog = Locale::parseCatalog(file_get_contents('locales/id/lomboklocale.json'));
$m = Locale::resolve('id', $catalog, 'ERR_NOT_FOUND', 'error.notFound',
                     ['name' => 'invoice.txt']);
echo $m->text;        // "invoice.txt tidak ditemukan" or "!!error.notFound!!" when missing
echo $m->fallback;    // false / true - missing translations stay visible, never blank
```

### Ordinal endings (1st, 2nd, 3rd ...)

```go
import lomboklocale "github.com/codinglombok/lomboklocale/go"

s, _ := lomboklocale.FormatMessage("en",
    "{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}",
    map[string]lomboklocale.Arg{"n": lomboklocale.Int(23)}) // "23rd"
```

### Picking the right plural for a free-form locale list

```rust
use lomboklocale::{plural_category, PluralCategory};
assert_eq!(plural_category("ar", "0")?, PluralCategory::Zero);
assert_eq!(plural_category("ar", "2")?, PluralCategory::Two);
assert_eq!(plural_category("hr", "5")?, PluralCategory::Other); // fixed in 0.2.0
```

### Validation without formatting

```ts
import { parseTag, canonicalize } from 'lomboklocale';
canonicalize('EN_latn-us-x-foo');  // "en-Latn-US-x-foo", throws LocaleError otherwise
```

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `BAD_NUMBER` with a value that looks fine | the value contains a group separator, `.5`, `1.` or an exponent above 1000 | pass plain decimal text (SPEC section 3); group separators are output-only |
| `BAD_ARGUMENT` on a `{n, plural, ...}` | the argument is a string that is not decimal text, or NaN/infinity | pass a number or decimal text; check the argument name |
| `!!some.message.id!!` in the output | the id is missing from the catalog or its pattern failed | the fallback marker is intentional (fail-visible); add the id to the catalog |
| `MISSING_OTHER` | a plural/select block has no `other` key | add `other{...}`; it is mandatory in the subset |
| Currency shows the ISO code instead of a symbol | the currency has no symbol in the CLDR subset | expected: `¤` falls back to the ISO code (14 currencies carry symbols) |
| Formatting in `xx` locale falls back to English digits/patterns | only 28 locales carry number/date data | plural rules still apply per locale; data for more locales can be added by re-running `scripts/extract_cldr.py` with an extended locale list |
| Windows output looks the same as Linux | yes - that is the contract | every port is byte-identical on every OS; differences are bugs |

*Document license: Apache-2.0 OR MIT. (c) codinglombok*
