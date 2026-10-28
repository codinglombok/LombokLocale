# LombokLocale — module map v0.2.0

Internal module map of the reference (Rust) port and the API surface it
feeds, plus the position of this repository inside the Lombok Ecosystem.

## Reference port module map (rust/src/)

```
lib.rs        re-exports, resolve(), LocalizedMessage, CLDR_VERSION
tag.rs        BCP 47: parse_tag, canonicalize, negotiate, data_locale, LanguageTag
plural.rs     plural_category, ordinal_category, PluralCategory (rule engine)
number.rs     format_number, NumberOptions, Style (pattern affixes, grouping)
date.rs       format_date, Date, DateStyle (pattern renderer)
decimal.rs    exact decimal text: parse, round (half away from zero), helpers
message.rs    format_message, Arg, Args, MAX_DEPTH (ICU MessageFormat subset)
catalog.rs    parse_catalog, Catalog (strict JSON, RFC 8259)
error.rs      Error, ErrorCode (16 codes), Result
tables.rs     rule-table data types (Rel, RuleTable, LocaleData)
data.rs       generated tables: PLURALS, ORDINALS, LOCALES, CURRENCY_DIGITS
```

Dependency direction inside the crate (arrows read "uses"):

```
lib.rs -> tag.rs -> tables.rs -> data.rs
lib.rs -> plural.rs -> decimal.rs
lib.rs -> number.rs -> decimal.rs
lib.rs -> date.rs
lib.rs -> message.rs -> number.rs, date.rs, plural.rs, decimal.rs
lib.rs -> catalog.rs -> error.rs
error.rs        (leaf: no internal deps)
```

The other ports mirror this layout with language-idiomatic file layout:
TypeScript and Python keep everything in one module plus a generated data
module; Go uses `locale.go` + generated `data.go`; PHP splits the classes
(`Locale`, `LanguageTag`, `Num`, `LocalizedMessage`, `LocaleError`,
`MessageParser`, `JsonReader`) plus generated `Data.php`. The vectors pin
all of them to the same behavior, so the module map is behavioral, not
structural.

## Data flow

```
cldr-json 47.0.0 --extract_cldr.py--> data/cldr47.json
data/cldr47.json --gen_ports.py--> rust/src/data.rs, typescript/src/data.ts,
    python/lomboklocale/_data.py, go/data.go, php/src/Data.php
hand-written cases + reference model (build_vectors.py)
    --> vectors/lomboklocale-vectors-v1.json --> all five port test suites
check_icu.mjs --> ICU (numbers, dates, plurals cross-check, skips on CLDR
    version mismatch)
```

## Ecosystem graph

This repository is part of the Lombok Ecosystem (github.com/codinglombok).
Within the ecosystem, repositories are runtime-independent: no repo depends
on another repo's code at build or run time (the doctor enforces "no
path/file dependencies between repositories"), and they share conventions
instead: one language folder per port, one shared deterministic vector file,
the doctor checker, the CI/release workflow shapes, and the staged release
ladder. The sibling repositories relevant to this one:

- LombokVector — the senior repository of the ecosystem and the calibration
  target for this repo's CI and release pipeline (its `ci.yml`, `release*.yml`,
  `scripts/lombok-doctor.sh`, `scripts/bump_version.sh` and docs naming are
  the patterns copied here).
- LombokLocale (this repository) — the localization core: BCP 47 negotiation,
  CLDR 47 plural/ordinal rules, number/currency/date formatting, an ICU
  MessageFormat subset, and JSON message catalogs. Any ecosystem project
  that needs user-facing text can adopt it directly from its registries
  (crates.io, npm, PyPI, Packagist, pkg.go.dev) with zero new runtime
  dependencies.

Consumers integrate through published packages only, so a version bump in
this repository never requires a matching commit elsewhere; the SPEC and the
vector file guarantee that upgrading a 0.2.x patch stays byte-compatible.

*Document license: Apache-2.0 OR MIT. (c) codinglombok*
