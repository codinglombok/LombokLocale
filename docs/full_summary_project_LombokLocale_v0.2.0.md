# LombokLocale — full project summary v0.2.0

Purpose, architecture, current status against the SPEC, measured metrics and
the roadmap, in one page. Companion documents: the
[SPEC](SPEC_LombokLocale_v0.2.0.md) (the contract), the
[API map](API_LombokLocale_v0.2.0.md) (the surface) and the
[distribution guide](how_to_dist_LombokLocale_v0.2.0.md) (the release path).

## Purpose

Applications that render user-facing text in more than one place - a browser,
a server, a batch job - get different numbers, dates and plural wording from
each stack's built-in formatter, because each ships a different CLDR version
with different defaults. An invoice total then differs between the frontend
and the backend, and "3 files" is spelled wrong in half of the locales.
LombokLocale pins the data (CLDR 47.0.0), pins the behavior (a normative SPEC
plus 342 deterministic vectors) and ships the exact same results as a
zero-dependency library in Rust, TypeScript, Python, Go and PHP. It is the
localization core of the Lombok Ecosystem: any project can adopt it from its
own registry and get identical text everywhere.

## Architecture

One reference implementation and four verified ports. The Rust crate owns
the contract; TypeScript, Python, Go and PHP are behaviorally identical by
construction because all five run the same 342-case vector file in CI and
the vectors are generated from an independent model, not from any port.

```
cldr-json 47.0.0 --extract--> data/cldr47.json --gen_ports--> embedded tables (5 ports)
reference model (Python, build_vectors.py) --> vectors/*.json --(one file)--> 5 test suites
                                              ICU cross-check (check_icu.mjs)
```

Key design decisions:

- Numbers are exact decimal text end to end; binary floats are converted
  once, to their shortest round-tripping decimal. `0.1 + 0.2` and `1e1000`
  are formatted exactly, half away from zero (SPEC section 3 and 5).
- The ICU MessageFormat subset covers the constructs real catalogs need
  (plural, select, selectordinal, offset, `=n`, number, date, apostrophe
  quoting, locale-formatted `#`); anything outside the subset fails with a
  stable code instead of guessing (fail-closed).
- Errors are part of the contract: 16 codes, compared by the vectors.
- The Rust crate is `no_std` + `alloc` with `#![forbid(unsafe_code)]`;
  no port has runtime dependencies.
- The data subset is exactly what the library uses (28 data locales, plural
  rules for 219 locales, ordinals for 104, 14 currencies) and its provenance
  and Unicode License v3 terms are recorded in `NOTICE`.

## Status vs the SPEC (0.2.0)

| Contract area | Status |
|---|---|
| BCP 47 parsing, canonicalization, RFC 4647 negotiation | implemented, vector-pinned (parse 30, negotiate 16, dataLocale 13 cases) |
| CLDR plural + ordinal rules | implemented from generated tables (92 cases), ICU cross-checked |
| Number / percent / currency formatting | implemented (83 cases) incl. Indian grouping, minimum grouping digits, currency spacing |
| Date formatting (4 styles, Gregorian, Latin digits) | implemented (34 cases) |
| MessageFormat subset + strict catalogs | implemented (message 51, catalog 18, resolve 5 cases), `MISSING_OTHER`, `TOO_DEEP` enforced |
| Error codes | 16 codes, identical in all ports |
| Security bounds (subtag 8, exponent 1000, depth 16, JSON depth 64) | enforced, tested |
| Known ICU differences | 4, documented in SPEC section 9 |

## Metrics (measured at 0.2.0)

| Metric | Value |
|---|---|
| Vector cases (shared, deterministic) | 342 (SHA-256 pinned in the SPEC and `vectors/SHA256SUMS`) |
| Ports passing the full vector file | 5 of 5 |
| TypeScript tests / coverage | 350 passed; 98.96% line, 96.36% branch, 100% functions |
| Python tests / coverage | 350 passed; 97% (branch mode) |
| Rust / Go / PHP suites | run per push in CI (cargo fmt/clippy/test + llvm-cov 90; gofmt/vet/test cover 90; php -l + run.php cover 90) |
| Runtime dependencies | 0 in every port |
| Code size (reference) | ~3,065 lines of Rust incl. generated tables |

## Roadmap

- 0.2.x: keep CLDR 47 pinned; port-level hardening already tracked as P0-P1
  in the backlog ([development_ide](development_ide_LombokLocale_v0.2.0.md)):
  `phpstan` at level max, stricter ruff/mypy baselines, more locales for the
  library's own catalogs (native-speaker review).
- 0.3.0 candidates: time-of-day formatting (fixed offsets only), more
  currency symbols, MessageFormat 2 evaluation - all gated by the
  SPEC-first protocol (SPEC + vectors + all five ports in one change).
- Non-goals stay non-goals: no timezone database, no locale data downloads at
  runtime, no full ICU parity (the four documented differences stand), no
  MessageFormat features beyond the subset.

*Document license: Apache-2.0 OR MIT. (c) codinglombok*
