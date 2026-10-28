# LombokLocale — language port map v0.2.0

One contract, five ports. Every port consumes the same
`vectors/lomboklocale-vectors-v1.json` (342 cases) in its own test suite,
so "passes CI" means "byte-identical to the SPEC". This file records the
role, location, entry points and platform notes of each port.

| Port | Role | Folder | Registry target | Vector runner |
|---|---|---|---|---|
| Rust | reference implementation; contract owner | `rust/` | crates.io `lomboklocale` | `rust/tests/vectors.rs` |
| TypeScript | required npm-port of the ecosystem | `typescript/` | npm `lomboklocale` | `typescript/tests/vectors.test.ts` |
| Python | scripting/data-science consumers | `python/` | PyPI `lomboklocale` | `python/tests/test_vectors.py` |
| Go | services and CLI consumers | `go/` | `github.com/codinglombok/lomboklocale/go` (tag `go/v*`) | `go/vectors_test.go` |
| PHP | server-side consumers | `php/` (manifest at repo root) | Packagist `codinglombok/lomboklocale` | `php/tests/vectors.php` |

## Rust — reference port

- Crate `lomboklocale`, MSRV 1.70, `no_std` + `alloc` (the `std` feature only
  adds `impl std::error::Error`), `#![forbid(unsafe_code)]`, zero runtime
  dependencies (`serde_json` is a test-only dev-dependency).
- Toolchain gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test`, MSRV `cargo check --lib` with and without default
  features, `cargo llvm-cov --fail-under-lines 90`.
- Publishing: `cargo publish` from the `crates` job of `release.yml` on the
  `v*` tag; staged rehearsal via `release-crates.yml`.

## TypeScript — the npm port

- Pure TypeScript, `tsc --strict` (also with `noUnusedLocals` and
  `noUnusedParameters`), zero runtime dependencies, Node 20/22/24 on all
  three operating systems.
- Dual ESM and CJS: `dist/index.js` (ESM), `dist/cjs/index.js` (CJS behind
  a `commonjs` marker), one `.d.ts` for both. `npm publish --provenance
  --access public` from the `npm` job; rehearsal via `release-npm.yml`.
- Notes: numbers passed as `number`/`bigint` are converted to their shortest
  round-tripping decimal text first (SPEC section 0.3); pass strings for
  exact control.

## Python — the PyPI port

- Flat typed module (`py.typed`), zero dependencies, Python 3.9+; build via
  `setuptools` (PEP 621 `pyproject.toml`), sdist + wheel, trusted publishing
  through the `pypi` environment (OIDC); rehearsal via `release-pypi.yml`.
- Gates: `ruff check --select E4,E7,E9,F .`, plain `mypy lomboklocale`
  (strict-mode findings are tracked as tech debt), `pytest` with branch
  coverage at >= 90%.

## Go — the pkg.go.dev port

- Module `github.com/codinglombok/lomboklocale/go`, Go 1.22+, zero
  dependencies, `gofmt` + `go vet`, table-driven tests; because Go versions
  subdirectory modules with git tags, `release.yml` pushes `go/v<version>`
  and pkg.go.dev picks the module up automatically.
- Note: Go never formats floats through binary parsing — `FloatString`
  produces the shortest exact decimal text first, so results stay
  byte-identical on every platform (no fused-multiply-add drift).

## PHP — the Packagist port

- Namespace `LombokLocale\`, PHP 8.1+, zero Composer dependencies; the
  manifest is the repository-root `composer.json` without a `version`
  field, so Packagist derives versions from the `v*` tags after the one-time
  `packagist.org/packages/submit` registration (auto-sync then follows every
  tag).
- Gates: `php -l` over `src/` and `tests/`, plus the pcov-instrumented
  `tests/run.php --coverage=90` harness; `phpstan` at level max is tracked
  as pre-release tech debt (see development_ide backlog P0).
- The test bootstrap registers a PSR-4 autoloader itself, so the suite runs
  without Composer installed.

## Port parity rules

1. All five ports MUST pass the full vector file before any `v*` tag; CI
   enforces this per push and per pull request.
2. Error codes are part of the contract; vectors compare codes only.
3. Naming follows the language (snake/camel/Pascal); the API document maps
   every function one-to-one across ports.
4. A missing port would stay fenced off in CI with a `hashFiles` guard
   rather than being deleted; today all five exist, so no guards are needed.

*Document license: Apache-2.0 OR MIT. (c) codinglombok*
