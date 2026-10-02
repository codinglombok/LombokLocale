#!/usr/bin/env python3
"""Extracts the CLDR 47 subset used by LombokLocale into data/cldr47.json.

Input: a checkout or download of unicode-org/cldr-json at tag 47.0.0
(only the files listed below are read). Usage:

    python3 scripts/extract_cldr.py /path/to/cldr-json/cldr-json

The output is committed; ports embed generated tables made from it by
scripts/gen_ports.py. CLDR data is (c) Unicode, Inc., Unicode License v3
(see LICENSE-UNICODE).
"""
import json
import re
import sys
import unicodedata
from pathlib import Path

CLDR_VERSION = "47.0.0"
LOCALES = ["en", "en-GB", "en-IN", "id", "ms", "jv", "su", "de", "fr", "es", "pt", "pt-PT", "it", "nl",
           "ru", "uk", "pl", "tr", "ar", "hi", "zh", "zh-Hant", "ja", "ko", "vi", "th", "sv", "cy"]
CURRENCIES = ["USD", "EUR", "GBP", "JPY", "CNY", "IDR", "MYR", "SGD", "INR", "KRW", "SAR", "AUD", "THB", "VND"]
CATEGORIES = ["zero", "one", "two", "few", "many"]


def load(root, rel):
    return json.loads((root / rel).read_text(encoding="utf-8"))


def is_s_or_z(ch):
    return unicodedata.category(ch)[0] in "SZ"


def parse_rule(text):
    """CLDR plural rule (samples stripped) -> list of OR groups, each a list of
    relations [operand, modulus, negated, [[lo, hi], ...]]."""
    text = text.split("@")[0].strip()
    if not text:
        return []
    groups = []
    for alt in text.split(" or "):
        rels = []
        for rel in alt.split(" and "):
            m = re.fullmatch(r"\s*([nivwftce])\s*(?:%\s*(\d+))?\s*(!=|=)\s*([\d.,\s]+)", rel)
            if not m:
                raise ValueError("cannot parse relation %r" % rel)
            operand, mod, op, ranges = m.group(1), int(m.group(2) or 0), m.group(3), m.group(4)
            rs = []
            for part in ranges.replace(" ", "").split(","):
                lo, _, hi = part.partition("..")
                rs.append([int(lo), int(hi or lo)])
            rels.append([operand, mod, 1 if op == "!=" else 0, rs])
        groups.append(rels)
    return groups


def plural_table(root, rel, key):
    data = load(root, rel)["supplemental"][key]
    out = {}
    for loc, rules in sorted(data.items()):
        entry = []
        for cat in CATEGORIES:
            r = rules.get("pluralRule-count-" + cat)
            if r is not None:
                entry.append([cat, parse_rule(r)])
        out[loc] = entry
    return out


def main():
    root = Path(sys.argv[1])
    out = {"cldr": CLDR_VERSION, "locales": LOCALES, "currencies": CURRENCIES}
    out["plurals"] = plural_table(root, "cldr-core/supplemental/plurals.json", "plurals-type-cardinal")
    out["ordinals"] = plural_table(root, "cldr-core/supplemental/ordinals.json", "plurals-type-ordinal")
    fr = load(root, "cldr-core/supplemental/currencyData.json")["supplemental"]["currencyData"]["fractions"]
    out["currencyDigits"] = {c: int(fr.get(c, fr["DEFAULT"])["_digits"]) for c in CURRENCIES}
    data = {}
    for loc in LOCALES:
        n = load(root, f"cldr-numbers-full/main/{loc}/numbers.json")["main"][loc]["numbers"]
        cur = load(root, f"cldr-numbers-full/main/{loc}/currencies.json")["main"][loc]["numbers"]["currencies"]
        g = load(root, f"cldr-dates-full/main/{loc}/ca-gregorian.json")["main"][loc]["dates"]["calendars"]["gregorian"]
        sym = n["symbols-numberSystem-latn"]
        symbols = {}
        for c in CURRENCIES:
            s = cur.get(c, {}).get("symbol", c)  # CLDR falls back to the ISO code
            symbols[c] = [s, 1 if is_s_or_z(s[0]) else 0, 1 if is_s_or_z(s[-1]) else 0]
        data[loc] = {
            "decimal": sym["decimal"], "group": sym["group"], "minus": sym["minusSign"],
            "percent": sym["percentSign"], "infinity": sym["infinity"], "nan": sym["nan"],
            "minGrouping": int(n["minimumGroupingDigits"]),
            "decimalPattern": n["decimalFormats-numberSystem-latn"]["standard"],
            "percentPattern": n["percentFormats-numberSystem-latn"]["standard"],
            "currencyPattern": n["currencyFormats-numberSystem-latn"]["standard"],
            "currencySymbols": symbols,
            "dateFormats": {k: g["dateFormats"][k] for k in ("full", "long", "medium", "short")},
            "months": [[g["months"]["format"][w][str(i)] for i in range(1, 13)] for w in ("abbreviated", "wide")],
            "days": [[g["days"]["format"][w][d] for d in ("sun", "mon", "tue", "wed", "thu", "fri", "sat")]
                     for w in ("abbreviated", "wide")],
            "era": g["eras"]["eraAbbr"]["1"],
        }
        sp = n["currencyFormats-numberSystem-latn"]["currencySpacing"]
        assert sp["beforeCurrency"]["insertBetween"] == "\xa0" and sp["afterCurrency"]["insertBetween"] == "\xa0"
        assert sp["beforeCurrency"]["currencyMatch"] == "[[:^S:]&[:^Z:]]", loc
    out["data"] = data
    Path(__file__).resolve().parent.parent.joinpath("data", "cldr47.json").write_text(
        json.dumps(out, ensure_ascii=False, indent=1, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
