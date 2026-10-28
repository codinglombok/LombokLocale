# LombokLocale — how to distribute v0.2.0

How to build, publish, verify and roll back a LombokLocale release. The
pipeline is the locked LombokVector pattern: publishing happens only from a
`v*` tag through `release.yml`, with per-registry staged rehearsals first.
Nothing here publishes silently; every path below is manual by design.

## Preconditions

1. CI is green on `main` (one job per port + the standards job), and
   `bash scripts/lombok-doctor.sh LombokLocale` passes.
2. Every manifest carries the same version, the top `CHANGELOG.md` entry is
   that version, and `docs/` contains only `*_LombokLocale_v<version>.md`
   files (the release `check-version` job re-checks all of this).
3. Registries have their one-time setup done:
   - npm: secret `NPM_TOKEN`; environment `npm` with a required reviewer.
   - crates.io: secret `CARGO_REGISTRY_TOKEN`; environment `crates`.
   - PyPI: trusted publishing for this repository with environment `pypi`
     (no token needed; OIDC).
   - Docker Hub (optional): secrets `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN`;
     GHCR needs nothing extra (`GITHUB_TOKEN`).
   - Packagist: submit the repository once at
     `packagist.org/packages/submit`; after that, every `v*` tag syncs
     automatically (the root `composer.json` has no `version` field).

## Staged rehearsals (always run these first)

Each registry has a `workflow_dispatch` workflow whose `dry_run` input
defaults to `true`: build + test + verification only. A real publish
requires switching `dry_run` to `false` AND passing the matching `tag`
input. Run them from the GitHub Actions tab on `main`:

1. "Release npm" (dry run): `npm ci`, `npm test`, `npm run build`,
   `npm pack --dry-run` with a tag/manifest consistency check.
2. "Release crates.io" (dry run): `cargo test`, `cargo publish --dry-run`.
3. "Release PyPI" (dry run): pytest, `python -m build`, `twine check`.
4. "Release docker" (dry run): build the image, load it, run the
   self-verifying smoke entrypoint (it prints one `ok, ...` line).

A dry run that passes proves the artifacts build and test; only then flip
`dry_run=false` for that registry, or go straight to the full release below.

## Full release (the `v*` tag path)

```bash
# 1. Bump every manifest and the docs naming (mechanical)
bash scripts/bump_version.sh X.Y.Z
# 2. Write the new top section of CHANGELOG.md and the matching
#    docs/changelog_LombokLocale_vX.Y.Z.md (the doctor checks both)
# 3. Local gates
bash scripts/lombok-doctor.sh LombokLocale
cd rust && cargo test && cargo clippy --all-targets -- -D warnings
cd ../typescript && npm ci && npm test && npm run build
cd ../python && python -m pytest && python -m mypy lomboklocale
cd ../go && go test ./...
cd ../php && php tests/run.php
# 4. Commit on main, tag, push (tags are never moved or deleted)
git commit -am "chore(release): vX.Y.Z"
git tag -a vX.Y.Z -m "LombokLocale X.Y.Z"
git push origin main vX.Y.Z
```

Pushing the tag triggers `release.yml`:

| Job | What it does | Gate |
|---|---|---|
| `check-version` | tag == every manifest == CHANGELOG top == doctor | blocking for all |
| `crates` | `cargo test` + `cargo publish` | environment `crates` |
| `npm` | `npm test` + `npm publish --provenance --access public` | environment `npm` |
| `pypi` | pytest + `python -m build` + trusted publish | environment `pypi` |
| `docker` | multi-arch image, GHCR always + Docker Hub when secrets exist | environment `docker` |
| `go-tag` | pushes `go/v<version>` for pkg.go.dev | - |
| `github-release` | `git archive` bundle + SHA256SUMS as release artifacts | environment `github-release` |

Reviewers on the environments hold each publish until approved, so the
release is staged by construction.

## Post-release verification

1. `npm view lomboklocale version` shows X.Y.Z;
   `npm view lomboklocale dist.tarball` then install it in a scratch project
   and run the quickstart from
   [guide_how_to_use_LombokLocale_v0.2.0.md](guide_how_to_use_LombokLocale_v0.2.0.md).
2. `cargo search lomboklocale` / crates.io page shows X.Y.Z with the
   Apache-2.0 OR MIT AND Unicode-3.0 license line.
3. `pip index versions lomboklocale` (or the PyPI page) shows X.Y.Z;
   `pip download lomboklocale==X.Y.Z` then import from the wheel.
4. `go list -m -versions github.com/codinglombok/lomboklocale/go` includes
   vX.Y.Z (allow a few minutes after the `go/v*` tag).
5. `docker run --rm ghcr.io/codinglombok/lomboklocale:vX.Y.Z` prints
   `ok, lomboklocale X.Y.Z (CLDR 47.0.0) ...` and exits 0.
6. The GitHub Release page shows the bundle + SHA256SUMS; verify with
   `sha256sum -c SHA256SUMS`.
7. Packagist shows X.Y.Z without any manual submit (auto-sync from the tag).

## Rollback / repair policy

- Tags are never moved or deleted; a bad artifact is superseded, not rewritten.
- npm/crates/PyPI: publish a corrected `X.Y.Z+1` (or yank on crates.io /
  deprecate on npm when the version is unusable).
- The Docker `latest` tag re-points automatically with the next release;
  pin consumers to `vX.Y.Z` tags.
- If a release environment blocks a job, fix forward on main, re-run the
  staged rehearsal for that registry, then re-tag; never force-push the tag.

*Document license: Apache-2.0 OR MIT. (c) codinglombok*
