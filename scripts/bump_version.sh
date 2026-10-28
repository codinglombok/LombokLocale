#!/usr/bin/env bash
# LombokLocale — staged version bump.
#
# Usage: scripts/bump_version.sh X.Y.Z
#
# Updates every manifest, the lock file, the Python version markers, renames
# the docs/*_v<old>.md files, and rewrites the version references. It does
# NOT write the CHANGELOG (human step), does NOT touch tags, commits, or git
# history, and never publishes anything.
set -euo pipefail

NEW="${1:?usage: bump_version.sh X.Y.Z}"
if ! [[ "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "ERROR: version must be MAJOR.MINOR.PATCH (got: $NEW)"
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OLD="$(sed -n 's/^  "version": *"\([^"]*\)".*/\1/p' typescript/package.json | head -1)"
if [ -z "$OLD" ]; then
  echo "ERROR: cannot read the current version from typescript/package.json"
  exit 1
fi
if [ "$OLD" = "$NEW" ]; then
  echo "Already at $NEW; nothing to do."
  exit 0
fi
echo "bump: $OLD -> $NEW"

# 1. manifests and code-level version markers
sed -i "s/\"version\": *\"${OLD}\"/\"version\": \"${NEW}\"/" typescript/package.json
sed -i "s/\"version\": *\"${OLD}\"/\"version\": \"${NEW}\"/g" typescript/package-lock.json
sed -i "s/^version = \"${OLD}\"/version = \"${NEW}\"/" rust/Cargo.toml python/pyproject.toml
sed -i "s/__version__ = \"${OLD}\"/__version__ = \"${NEW}\"/" python/lomboklocale/__init__.py
sed -i "s/ll\.__version__ == \"${OLD}\"/ll.__version__ == \"${NEW}\"/" python/tests/test_api.py

# 2. documentation: rename the versioned files, then update version references
#    inside them and in the port READMEs / source doc comments.
for f in docs/*_v"${OLD}".md; do
  [ -e "$f" ] || continue
  n="${f/_v${OLD}.md/_v${NEW}.md}"
  git mv "$f" "$n"
  sed -i "s/v${OLD}/v${NEW}/g; s/${OLD}\.x/${NEW}\.x/g; s/LombokLocale ${OLD}/LombokLocale ${NEW}/g; s/| SPEC version | ${OLD} /| SPEC version | ${NEW} /; s/| Version | ${OLD} |/| Version | ${NEW} |/" "$n"
done
for f in README.md Dockerfile go/README.md php/README.md python/README.md rust/README.md \
         typescript/README.md php/src/Locale.php python/lomboklocale/__init__.py \
         rust/src/lib.rs typescript/src/index.ts; do
  [ -e "$f" ] && sed -i "s/v${OLD}/v${NEW}/g" "$f"
done
sed -i "s/^ARG VERSION=${OLD}/ARG VERSION=${NEW}/" Dockerfile
sed -i "s/e\.g\. v${OLD}/e.g. v${NEW}/g" .github/workflows/release-npm.yml \
  .github/workflows/release-crates.yml .github/workflows/release-pypi.yml \
  .github/workflows/release-docker.yml

echo
echo "Mechanical bump done. Remaining human steps:"
echo "  1. Write the new top section in CHANGELOG.md (must be [${NEW}]; the"
echo "     doctor checks it) and the matching row in"
echo "     docs/changelog_LombokLocale_v${NEW}.md."
echo "  2. If the contract changed: update docs/SPEC_LombokLocale_v${NEW}.md"
echo "     FIRST, rebuild the vectors, refresh vectors/SHA256SUMS, and port"
echo "     the change to every runner (SPEC-first protocol)."
echo "  3. bash scripts/lombok-doctor.sh LombokLocale"
echo "     plus the port test suites (see README Development section)."
echo "  4. Commit on main, then: git tag -a v${NEW} -m \"...\""
echo "     git push origin v${NEW}   (release.yml publishes every registry)"
