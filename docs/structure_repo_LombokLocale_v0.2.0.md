# LombokLocale — repository structure v0.2.0

Annotated directory tree of the repository at 0.2.0. Language ports live in
their own top-level folders (the LombokVector convention; there is
deliberately no `ports/` directory), one shared vector file drives all five
ports, and every generated artifact has exactly one generator so CI can
refuse stale copies.

## Tree

```
LombokLocale/
├── .github/workflows/        CI + release, one job per port (ci.yml, release*.yml)
├── data/
│   └── cldr47.json           generated: CLDR 47.0.0 subset (scripts/extract_cldr.py)
├── docs/                     public document set, English, v0.2.0 only
│   ├── API_LombokLocale_v0.2.0.md
│   ├── SPEC_LombokLocale_v0.2.0.md
│   ├── changelog_LombokLocale_v0.2.0.md
│   ├── map_LombokLocale_v0.2.0.md
│   ├── structure_repo_LombokLocale_v0.2.0.md
│   ├── full_summary_project_LombokLocale_v0.2.0.md
│   ├── guide_how_to_use_LombokLocale_v0.2.0.md
│   ├── how_to_dist_LombokLocale_v0.2.0.md
│   ├── development_ide_LombokLocale_v0.2.0.md
│   └── Lang_LombokLocale_v0.2.0.md
├── locales/                  this library's own message catalogs (not CLDR)
│   ├── STATUS.md             review status per language
│   └── <tag>/lomboklocale.json   en (source), id (reviewed), 8 drafts
├── rust/                     reference port (crates.io crate)
│   ├── Cargo.toml            manifest, MSRV 1.70, no_std + alloc
│   ├── src/                  lib, tag, plural, number, date, decimal, message,
│   │                         catalog, error, tables + data.rs (generated)
│   └── tests/                api.rs, catalogs.rs, vectors.rs
├── typescript/               npm package, dual ESM + CJS
│   ├── package.json / package-lock.json / tsconfig*.json
│   ├── scripts/              coverage.mjs (custom runner), write-cjs-marker.mjs
│   ├── src/                  index.ts (manual), data.ts (generated)
│   ├── tests/                api.test.ts, vectors.test.ts
│   └── dist/ + build/        build outputs (gitignored)
├── python/                   PyPI package (PEP 621)
│   ├── pyproject.toml
│   ├── lomboklocale/         __init__.py (manual), _data.py (generated), py.typed
│   └── tests/                test_api.py, test_vectors.py
├── go/                       Go module github.com/codinglombok/lomboklocale/go
│   ├── go.mod                go 1.22, module tag go/v0.2.0
│   ├── locale.go (manual)    data.go (generated)
│   └── api_test.go, vectors_test.go
├── php/                      Packagist package (manifest at the repository root)
│   ├── src/                  Locale, LanguageTag, Num, LocalizedMessage,
│   │                         LocaleError, MessageParser, JsonReader (manual)
│   │                         Data.php (generated)
│   └── tests/                api.php, vectors.php, run.php, bootstrap.php
├── vectors/
│   ├── lomboklocale-vectors-v1.json  generated: 342 cases (build_vectors.py)
│   ├── SHA256SUMS                    checksum of the vector file
│   ├── build_vectors.py              deterministic generator + reference model
│   └── check_icu.mjs                 ICU cross-check (skips on CLDR mismatch)
├── scripts/
│   ├── extract_cldr.py      data/cldr47.json from cldr-json 47.0.0
│   ├── gen_ports.py         port tables from data/cldr47.json (--check in CI)
│   ├── lombok-doctor.sh     ecosystem standards checker (v3.6)
│   ├── bump_version.sh      staged version bump
│   └── docker_smoke.py      container image entrypoint smoke test
├── .dockerignore             build-context hygiene
├── Dockerfile                Python port + self-verifying entrypoint image
├── composer.json             PHP manifest at the ROOT (no version field)
├── NOTICE                    embedded CLDR provenance and license record
├── CHANGELOG.md / README.md / LICENSE-APACHE / LICENSE-MIT / LICENSE-UNICODE
└── .gitignore                build outputs + internal-document block (ADR-024)
```

## Generated versus manual

| Artifact | Generator | Verified by |
|---|---|---|
| `data/cldr47.json` | `scripts/extract_cldr.py` (input: cldr-json tag 47.0.0) | regenerated on demand; source recorded in `NOTICE` |
| `rust/src/data.rs`, `typescript/src/data.ts`, `python/lomboklocale/_data.py`, `go/data.go`, `php/src/Data.php` | `scripts/gen_ports.py` | `python3 scripts/gen_ports.py --check` in the CI standards job |
| `vectors/lomboklocale-vectors-v1.json` | `vectors/build_vectors.py` | rebuild + `git diff --exit-code` + `sha256sum -c vectors/SHA256SUMS` in CI |
| everything else | hand-written | port test suites + vectors + doctor |

Internal planning documents (masterplan, architecture, tech-debt) are local
only and never committed; the `.gitignore` v3 block keeps them out of the
repository and the release bundle (built with `git archive`, tracked files
only).

## Conventions worth knowing before editing

- SPEC first: when behavior changes, `docs/SPEC_LombokLocale_v<version>.md`,
  the vector file, and all five ports change in the same commit; the
  `check-version` release job refuses a tag that does not match every
  manifest, and the doctor refuses mismatched docs.
- One port per top-level folder; the CI has one job per folder with
  `defaults.run.working-directory`, exactly like LombokVector.
- The PHP manifest sits at the repository root because Packagist only reads
  the root `composer.json`; its version comes from the git tag.
