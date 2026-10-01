# LombokLocale

Zero-dependency i18n core: BCP-47 negotiation, CLDR plural/ordinal rules, a MessageFormat 2-lite runtime, locale-aware number/date/currency formatting, and message catalogs.

A standalone, general-purpose library of the **Lombok Ecosystem** — Tier **L0**. No mandatory dependency on any other Lombok library (L0).

> **Universal by design.** Usable by anyone — from small embedded devices to premium industrial software — without any application or framework. It is not part of, and not owned by, any app or server (e.g. RAG stacks); apps are merely example users. 

## Status

🔵 **v0.1.0 — not yet published** to GitHub/registries. Rust reference + **TypeScript port** both pass the same shared test vectors (`vectors/lomboklocale-vectors-v1.json`, byte-identical behaviour per ADR-015). Python/Go/PHP ports are stubs. Plural rules are hand-written from CLDR; the CLDR release they track is not yet pinned (see docs).

## Features

- **BCP-47** parse + RFC 4647 negotiation (`zh-Hant-TW` → `zh-Hant` → `zh-TW` → `zh`)
- **CLDR plural (cardinal) and ordinal rules** (subset): en/Germanic, fr/pt, ru/Slavic, ar (6 categories), cy, no-plural languages (`id`, `ja`, `zh`, …)
- **MessageFormat 2-lite**: `{name}`, `{n, plural, …}`, `{n, selectordinal, …}`, `{v, select, …}`, `#`; nesting depth capped
- **Number / currency / date** formatting per locale; `NaN`/∞/huge values handled; strict ISO dates (leap years honoured)
- **Message catalogs** `locales/<bcp47>/<repo>.json` (strict flat JSON) + `resolve()` with the `code` + `messageId` contract; 10 catalogs (en, id + 8 draft machine translations — see `locales/STATUS.md`)
- `no_std + alloc` core, zero dependencies

## Quick Start

### Rust (reference)

```toml
[dependencies]
lomboklocale = { git = "https://github.com/codinglombok/LombokLocale", package = "lomboklocale" }   # not on crates.io yet
```

```rust
use lomboklocale::*;
negotiate(&["id-ID", "en"], &["en", "id"], "en");                          // "id"
let args = [("n", ArgValue::UInt(2))];
format_message("en", "{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}", &args).unwrap(); // "2nd"
format_currency(15000.0, "IDR", "id");                                      // "Rp15.000,00"
let cat = parse_catalog(r#"{"hi": "Halo, {name}!"}"#).unwrap();
resolve("id", &cat, "GREET", "hi", &[("name", ArgValue::Str("Bali".into()))]).text; // "Halo, Bali!"
```

### TypeScript

```bash
cd typescript && npm install && npm test     # builds, then runs every shared vector
```

Zero runtime dependencies, ESM, Node ≥ 18. See `docs/API_LombokLocale_v0.1.0.md` for the camelCase API.

## Testing

```bash
cd rust && cargo test --release                       # unit + robustness (pseudo-fuzz) + shared vectors
cd rust && cargo build --no-default-features          # no_std + alloc proof
cd typescript && npm test                             # same vectors, TypeScript port
LOMBOK_REGEN=1 cargo test --release --test vectors    # regenerate expected outputs from the Rust reference (review the diff!)
./scripts/lombok-doctor-docs.sh LombokLocale                # 12 docs, versions, vector hash, license, no ownership claims
```

Vector inputs are authored in `vectors/gen_inputs.py` (deterministic); expected outputs come from the Rust reference and are reviewed by hand and, where possible, by independent checks. Changing any vector requires updating its SHA-256 in `docs/SPEC_LombokLocale_v0.1.0.md` (CI enforces it).

## License

Dual-licensed under **Apache-2.0 OR MIT**, at your option — see [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).
