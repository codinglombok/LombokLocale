#!/usr/bin/env python3
"""Self-verifying smoke test for the LombokLocale container image.

Runs a fixed set of SPEC examples through the Python port (byte-exact
expectations, including the U+00A0 and U+202F separators CLDR specifies) and
prints one line on success. Used as the Docker image entrypoint and by the
release-docker dry run; exit status is 0 only when every check passes.

This file is part of the repository, not of the published packages.
"""

import sys

import lomboklocale as ll

CASES = [
    ("negotiate", lambda: ll.negotiate(["id-ID", "en"], ["en", "id"], "en"), "id"),
    ("format_number decimal", lambda: ll.format_number("id", "1234567.891"),
     "1.234.567,891"),
    ("format_number currency", lambda: ll.format_number("de", "15000", style="currency", currency="EUR"),
     "15.000,00\u00a0\u20ac"),
    ("format_number Indian grouping", lambda: ll.format_number("hi", "1234567.5"),
     "12,34,567.5"),
    ("format_number French currency", lambda: ll.format_number("fr", 1234.5, style="currency", currency="EUR"),
     "1\u202f234,50\u00a0\u20ac"),
    ("format_date full", lambda: ll.format_date("id", "2026-10-02", "full"),
     "Jumat, 02 Oktober 2026"),
    ("format_date Japanese", lambda: ll.format_date("ja", "2026-10-02", "full"),
     "2026\u5e7410\u67082\u65e5\u91d1\u66dc\u65e5"),
    ("plural_category", lambda: ll.plural_category("pl", 21), "many"),
    ("format_message plural", lambda: ll.format_message(
        "en", "{n, plural, one{# file} other{# files}}", {"n": 1234}),
     "1,234 files"),
    ("format_message selectordinal", lambda: ll.format_message(
        "en", "{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}", {"n": 23}),
     "23rd"),
    ("resolve fallback", lambda: ll.resolve(
        "en", ll.parse_catalog("{}"), "ERR_X", "missing.id", {}).text,
     "!!missing.id!!"),
    ("parse_catalog", lambda: ll.parse_catalog('{"greeting": "Halo {name}"}')["greeting"],
     "Halo {name}"),
]

failures = []
for name, fn, expected in CASES:
    got = fn()
    if got != expected:
        failures.append(f"{name}: got {got!r}, want {expected!r}")

# Errors are fail-closed and carry a stable [CODE]; the code must match the SPEC.
try:
    ll.parse_tag("")
    failures.append("parse_tag(''): no exception raised, want LocaleError EMPTY")
except ll.LocaleError as exc:
    if str(exc) != "EMPTY":
        failures.append(f"parse_tag(''): got {exc}, want EMPTY")

if failures:
    for f in failures:
        print(f"smoke FAIL: {f}", file=sys.stderr)
    sys.exit(1)

print(f"ok, lomboklocale {ll.__version__} (CLDR {ll.CLDR_VERSION}) python {sys.version.split()[0]}")
