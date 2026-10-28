# LombokLocale — development ideas & backlog v0.2.0

The living backlog of this repository: priorities, done-criteria and
non-goals. P0 items are release blockers for the `v0.2.0` tag; P1 items are
follow-ups for the 0.2.x line; P2/P3 are candidates and research. Items move
up when they get a written decision and a criterion, never by accident.

## P0 - before the v0.2.0 tag

| Item | Done when |
|---|---|
| phpstan (level max) over `php/src` | a `phpstan.neon` at level max passes locally and a `phpstan` step runs green in the `php` CI job |
| Native-speaker review of the 8 machine-translated catalogs (`es fr de pt ru ja ko zh-Hans`) | each `locales/<tag>/lomboklocale.json` reviewed by a native speaker, `locales/STATUS.md` updated, `rust/tests/catalogs.rs` still green |
| Registry one-time setup | `NPM_TOKEN`, `CARGO_REGISTRY_TOKEN`, PyPI trusted publishing, environments with reviewers (`npm`, `pypi`, `crates`, `docker`, `github-release`), Packagist submit |
| Release rehearsal | all four `release-*.yml` dry runs pass on `main` |

## P1 - 0.2.x follow-ups

| Item | Done when |
|---|---|
| Stricter Python lint baseline | `ruff check` with the default rule set passes (today: 41 findings recorded in the tech-debt file; currently gated at `E4,E7,E9,F`) |
| Stricter Python typing | `mypy --strict lomboklocale` passes (today: 29 strict findings; currently gated plain) |
| MSRV bump evaluation | decide whether Rust MSRV stays 1.70 or follows ecosystem policy upward; recorded decision either way |
| `check_icu.mjs` on pinned ICU | run the cross-check in CI against an ICU build with CLDR 47 (currently it skips on CLDR 48 machines) |
| More own-library locales (`ar`, `hi`, `tr`, `vi`, `ms`, `jv`, `su` ...) | native-reviewed catalogs, keys identical to `en`, catalogs test green |

## P2 - 0.3.0 candidates (all SPEC-first)

| Item | Notes |
|---|---|
| Time-of-day formatting | fixed UTC offsets only, CLDR time patterns; no timezone database |
| MessageFormat 2 (LDML 47) evaluation | a separate mode, not a break of the ICU subset |
| More currency symbols | extend the 14-currency subset; regenerate ports via `extract_cldr.py` + `gen_ports.py` |
| Compact notation / significant digits | new style; needs vectors for all five ports |
| CLI smoke binary | a small `cargo` example + npm bin entry running the quickstart |

## P3 - research

- Runtime catalog loading patterns (streamed vs embedded) without adding
  file I/O to the core (SPEC section 10 keeps the core I/O-free).
- A sixth port (e.g. C via the Rust crate) - only if an ecosystem project
  actually needs it; the CI hashFiles guard pattern stays available.
- Fuzzing harnesses over `parse_tag`, `format_message` and `parse_catalog`.

## Non-goals

- No runtime downloads of locale data; everything is embedded.
- No timezone database, no calendar systems beyond Gregorian, no non-Latin
  digit output (the `-u-nu-*` extension is ignored by design).
- No full ICU/CLDR parity: the four documented differences in SPEC section 9
  are contract, not bugs.
- No runtime dependency in any port, ever; dev/test tooling is allowed in
  CI only.
- No feature that cannot be vectorized deterministically across all five
  ports.

## How to contribute an item

1. Write the done-criterion first (measurable, testable).
2. If it changes behavior: SPEC first, then vectors, then all five ports in
   one change (the CI standards job rejects stale vectors or tables).
3. Keep the artifacts English; run
   `bash scripts/lombok-doctor.sh LombokLocale` before opening the pull
   request.

*Document license: Apache-2.0 OR MIT. (c) codinglombok*
